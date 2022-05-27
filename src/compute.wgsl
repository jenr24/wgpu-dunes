[[block]]
struct InputBuffer {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> input: InputBuffer;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    input.data[global_id.x] = input.data[global_id.x] + 42.0;
}

