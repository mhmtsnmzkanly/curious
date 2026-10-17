/* Bounded, read-only genealogy inspection using the existing query API. */
(function (root) {
  "use strict";
  function validId(value) {
    const id = Number(value);
    if (!Number.isSafeInteger(id) || id < 1) throw new Error("Geçerli bir birey numarası gir.");
    return id;
  }
  class Reader {
    constructor(api, id, hint = null) {
      this.api = api;
      this.id = validId(id);
      this.hint = hint;
      this.shards = null;
      this.index = 0;
      this.before = null;
      this.individual = null;
      this.lastTick = null;
      this.latestTrace = null;
      this.death = null;
      this.children = new Map();
      this.parents = hint?.parents || [];
      this.page = [];
      this.done = false;
      this.scanned = 0;
      this.familyLimited = false;
    }
    async more() {
      if (this.done) return;
      if (!this.shards) {
        const inventory = await this.api("/databases");
        this.shards = inventory.map((s) => validId(Number(s.shard) + 1) - 1).sort((a,b) => b-a);
      }
      this.page = [];
      // Empty shards are bounded too. Opening old gzip archives can take time.
      for (let attempt = 0; attempt < 4 && this.index < this.shards.length; attempt++) {
        const shard = this.shards[this.index];
        const limit = this.before === null ? "" : `AND d.record_id < ${this.before}`;
        const hintRecord = this.hint?.shard === shard ? validId(this.hint.record) : "NULL";
        const point = `COALESCE((SELECT MAX(record_id) FROM batch),${hintRecord})`;
        const sql = `WITH batch AS (
          SELECT d.record_id,json_remove(d.trace_json,'$.perception') AS trace
          FROM decision_traces d JOIN records r ON r.id=d.record_id
          WHERE d.entity_id=${this.id} AND r.kind='tick' ${limit}
          ORDER BY d.record_id DESC LIMIT 50
        ) SELECT
          (SELECT json_group_array(json_object('record',record_id,'trace',json(trace))) FROM batch),
          (SELECT e.value FROM records r,json_each(r.state_json,'$.entities') e
           WHERE r.id=${point} AND json_extract(e.value,'$.id')=${this.id} LIMIT 1),
          (SELECT json_object('tick',r.tick,'event',json(e.payload)) FROM events e
           JOIN records r ON r.id=e.record_id WHERE e.record_id BETWEEN ${point} AND ${point}+32
           AND e.kind='EntityDied' AND json_extract(e.payload,'$.EntityDied.id')=${this.id}
           ORDER BY e.record_id,e.ordinal LIMIT 1)`;
        const result = await this.api("/query", {shard, sql});
        if (result.truncated || !result.rows.length) throw new Error("Arşiv yanıtı eksik; tarama tamamlanmadı.");
        const [raw, body, death] = result.rows[0];
        const rows = JSON.parse(raw || "[]").sort((a,b) => b.record-a.record);
        if (!this.individual && body) {
          this.individual = JSON.parse(body);
          this.parents = this.individual.parents || [];
          if (!rows.length && this.hint?.born != null) this.lastTick = this.hint.born;
        }
        if (!this.death && death) this.death = JSON.parse(death);
        // A death at a shard boundary may be in the immediately newer archive.
        if (!this.death && this.lastTick === null && rows.length && this.index > 0) {
          const newer = this.shards[this.index-1];
          const boundary = await this.api("/query", {shard:newer, sql:`SELECT r.tick,e.payload FROM events e JOIN records r ON r.id=e.record_id WHERE e.record_id <= (SELECT MIN(id)+32 FROM records) AND e.kind='EntityDied' AND json_extract(e.payload,'$.EntityDied.id')=${this.id} ORDER BY e.record_id,e.ordinal LIMIT 1`});
          if (boundary.rows.length) this.death = {tick:boundary.rows[0][0],event:JSON.parse(boundary.rows[0][1])};
        }
        for (const row of rows) {
          const trace = row.trace;
          if (this.lastTick === null) this.lastTick = trace.tick;
          if (!this.latestTrace) this.latestTrace = trace;
          this.page.push(trace);
          for (const event of trace.outcome?.events || []) {
            const mating = event.EntityMated;
            if (mating && (mating.parent1 === this.id || mating.parent2 === this.id)) {
              if (this.children.size < 200 || this.children.has(mating.child)) {
                this.children.set(mating.child,{id:mating.child,parents:[mating.parent1,mating.parent2],shard,record:row.record,born:trace.tick});
              } else this.familyLimited = true;
            }
            if (event.EntityDied?.id === this.id) this.death = {tick:trace.tick,event};
          }
        }
        if (rows.length < 50) {
          this.index++;
          this.before = null;
          this.scanned++;
        } else this.before = validId(rows[rows.length-1].record);
        if (rows.length) break;
      }
      this.done = this.index >= this.shards.length;
    }
  }
  const exported = { Reader, validId };
  if (typeof module !== "undefined" && module.exports) module.exports = exported;
  else root.CuriousArchive = exported;
})(typeof window === "undefined" ? globalThis : window);
