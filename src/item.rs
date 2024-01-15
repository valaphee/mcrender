use std::f32::consts::FRAC_PI_4;

use wgpu::util::DeviceExt;

use crate::{Renderer, Vertex};

#[derive(serde::Serialize, serde::Deserialize)]
struct Query {
    nbt: Option<String>,
}

fn vertex(position: [i8; 3]) -> Vertex {
    Vertex {
        position: [
            position[0] as f32,
            position[1] as f32,
            position[2] as f32,
            1.0,
        ],
    }
}

#[actix_web::get("/item/{namespace}/{key}.png")]
async fn get(
    renderer: actix_web::web::Data<Renderer>,
    _path: actix_web::web::Path<(String, String)>,
    _query: actix_web::web::Query<Query>,
) -> impl actix_web::Responder {
    let vertices = &[
        // top (0, 0, 1)
        vertex([-1, -1, 1]),
        vertex([1, -1, 1]),
        vertex([1, 1, 1]),
        vertex([-1, 1, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1]),
        vertex([1, 1, -1]),
        vertex([1, -1, -1]),
        vertex([-1, -1, -1]),
        // right (1, 0, 0)
        vertex([1, -1, -1]),
        vertex([1, 1, -1]),
        vertex([1, 1, 1]),
        vertex([1, -1, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1]),
        vertex([-1, 1, 1]),
        vertex([-1, 1, -1]),
        vertex([-1, -1, -1]),
        // front (0, 1, 0)
        vertex([1, 1, -1]),
        vertex([-1, 1, -1]),
        vertex([-1, 1, 1]),
        vertex([1, 1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1]),
        vertex([-1, -1, 1]),
        vertex([-1, -1, -1]),
        vertex([1, -1, -1]),
    ];
    let indices: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

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

    let projection = glam::Mat4::perspective_rh(FRAC_PI_4, 1.0, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    let projection_view = projection * view;

    let uniform_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(projection_view.as_ref()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
    let bind_group = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &renderer.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
    let pipeline = renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&renderer.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &renderer.shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
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
                targets: &[Some(output_texture.format().into())],
            }),
            multiview: None,
        });

    let vertex_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let index_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
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
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
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
