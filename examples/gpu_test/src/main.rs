use mewo_galaxy::prelude::*;
use mewo_gpu::prelude::*;
use mewo_window::prelude::*;

fn main() {
    let mut galaxy = Galaxy::new();

    window_init(&galaxy);
    galaxy.update();

    let window = galaxy
        .get_resource::<Window, _>(Window::single_resource())
        .unwrap();
    let mut context = GpuContext::new(
        &window.get_raw_display(),
        &window.get_raw_window(),
        window.get_width(),
        window.get_height(),
    )
    .unwrap();
    drop(window);

    let vs = include_bytes!("vs.spv");
    let fs = include_bytes!("fs.spv");

    let shaders =
        GpuShaderSet::shaders(&[(GpuShaderType::Vertex, vs), (GpuShaderType::Fragment, fs)]);
    let program = context.new_program(&shaders).unwrap();

    #[rustfmt::skip]
    let vertex_data: Vec<GpuVertexBufferElement> = vec![
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];

    #[rustfmt::skip]
    let index_data: Vec<GpuIndexBufferElement> = vec![
        0, 1, 2
    ];

    let vbo = context
        .new_vertex_buffer(&vertex_data, GpuBufferStorageType::Static)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, GpuBufferStorageType::Static)
        .unwrap();

    let mut seq_builder = GpuSequenceBuilder::new();
    let color_pass = seq_builder.pass(program);
    color_pass.add_vertex_buffer(vbo);
    color_pass.set_index_buffer(ibo);
    let color_pass_ref = color_pass.write_color(0, true, GpuAttachmentLoadOp::Clear);
    let sequence = context
        .compile_sequence(seq_builder.get_passes(), GpuCompositeLayer::Primary)
        .unwrap();

    loop {
        let mut submit = GpuSubmit::new();

        let mut color_pass_data = GpuPassSubmitData::new();
        color_pass_data.draw_indexed(0, index_data.len());

        let mut sequence_data = GpuSequenceSubmitData::new(sequence);
        sequence_data
            .pass(color_pass_data)
            .set_attachment_clear_color(
                color_pass_ref,
                GpuClearColor {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            );

        submit.sequence(sequence_data).present();

        context.submit(submit).unwrap();

        window_update(&galaxy);
        galaxy.update();
    }
}
