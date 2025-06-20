struct Uniforms {
    transform: mat4x4<f32>,
    transform_normals: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.normal = (uniforms.transform_normals * vec4<f32>(in.normal, 0.0)).xyz;
    out.position = uniforms.transform * vec4<f32>(in.position, 1.0);
    // We use premultiplied alpha blending.
    out.color = vec4<f32>(in.color.rgb * in.color.a, in.color.a);

    return out;
}

const pi: f32 = 3.14159265359;

@fragment
fn frag_face(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return out;
}

@fragment
fn frag_model(in: VertexOutput) -> FragmentOutput {
    let light = vec3<f32>(0.0, 0.0, -1.0);

    // normal-based shading
    let angle = acos(dot(light, -in.normal));
    let f_angle = angle / (pi * 0.75);
    let f_normal = max(1.0 - f_angle, 0.0);

    // Depth-based shading to improve visibility of blind holes and recessed surfaces
    let depth = in.position.z;
    
    //50% max darkening
    let depth_factor = 1.0 - depth * 0.5;
    
    let final_shading = f_normal * depth_factor;

    var out: FragmentOutput;
    out.color = vec4<f32>(in.color.rgb * final_shading, in.color.a);

    return out;
}

@fragment
fn frag_mesh(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0 - in.color.rgb, in.color.a);
    return out;
}
