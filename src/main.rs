use curious::set_global_seed_with_time;

fn main() {
    // RNG için seed'i zaman damgası olarak günceller
    set_global_seed_with_time();
}
