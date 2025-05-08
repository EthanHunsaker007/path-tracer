use ray_tracer::run;

fn main() {
    pollster::block_on(run());
}