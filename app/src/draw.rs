use macroquad::prelude::*;

pub fn kf_draw_texture(
    texture: &Texture2D,
    x: f32,
    y: f32,
    color: Color,
    params: DrawTextureParams,
) {
    draw_texture_ex(texture, x, y, color, params);
    //     let context = unsafe { get_internal_gl() };

    //     let [width, height] = texture.size().to_array();

    //     let Rect { x: mut sx, y: mut sy, w: mut sw, h: mut sh } =
    //         params.source.unwrap_or(Rect { x: 0., y: 0., w: width, h: height });

    //     let (w, h) = match params.dest_size {
    //         Some(dst) => (dst.x, dst.y),
    //         _ => (sw, sh),
    //     };

    //     let p = [vec2(x, y), vec2(x + w, y), vec2(x + w, y + h), vec2(x, y + h)];

    //     // offsetting the uv slightly inward solves the problem of seams
    //     // it only happened at some dpi in the web build for me, but was still annoying

    //     const OFFSET: f32 = 1.0 / 128.0;
    //     sx += OFFSET;
    //     sy += OFFSET;
    //     sh -= OFFSET * 2.;
    //     sw -= OFFSET * 2.;

    //     #[rustfmt::skip]
    //     let vertices = [
    //         Vertex::new(p[0].x, p[0].y, 0.,  sx      /width,  sy      /height, color),
    //         Vertex::new(p[1].x, p[1].y, 0., (sx + sw)/width,  sy      /height, color),
    //         Vertex::new(p[2].x, p[2].y, 0., (sx + sw)/width, (sy + sh)/height, color),
    //         Vertex::new(p[3].x, p[3].y, 0.,  sx      /width, (sy + sh)/height, color),
    //     ];
    //     let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    //     context.quad_gl.texture(Some(&texture));
    //     context.quad_gl.draw_mode(DrawMode::Triangles);
    //     context.quad_gl.geometry(&vertices, &indices);
}
