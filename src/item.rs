use wgpu::util::DeviceExt;
use crate::Renderer;

#[derive(serde::Serialize, serde::Deserialize)]
struct Nbt {
    nbt: Option<String>,
}

#[actix_web::get("/item/{namespace}.{key}.png")]
async fn get(renderer: actix_web::web::Data<Renderer>, _id: actix_web::web::Path<(String, String)>, _nbt: actix_web::web::Query<Nbt>) -> impl actix_web::Responder {
    println!("Loading {} {} {:?}", _id.0, _id.1, _nbt.nbt);

    let output_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 128,
            height: 128,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let output_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 128 * 128 * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &[0u8; 4 * 4 * 4],
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM
    });
    let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &renderer.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding()
            }
        ],
    });
    let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&renderer.pipeline_layout),
        vertex: wgpu::VertexState {
            module: &renderer.shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &renderer.shader,
            entry_point: "fs_main",
            targets: &[None],
        }),
        multiview: None,
    });

    let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations::default(),
            })],
            ..Default::default()
        });
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        //render_pass.set_index_buffer();
        //render_pass.set_vertex_buffer();
        //render_pass.draw_indexed();
    }
    encoder.copy_texture_to_buffer(
        output_texture.as_image_copy(),
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(128 * 4),
                rows_per_image: None,
            },
        },
        output_texture.size(),
    );
    renderer.queue.submit(std::iter::once(encoder.finish()));

    let mut data = vec![];
    {
        let mut encoder = png::Encoder::new(&mut data, 128, 128);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
        renderer.device.poll(wgpu::Maintain::Wait);

        let buffer_view = buffer_slice.get_mapped_range();
        writer.write_image_data(&buffer_view).unwrap();
    }

    actix_web::HttpResponse::Ok()
        .content_type("image/png")
        .body(data)
}
