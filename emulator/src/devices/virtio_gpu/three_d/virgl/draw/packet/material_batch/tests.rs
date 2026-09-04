use super::packet;
use super::super::super::{DrawMaterial, DrawWork};
use super::super::super::super::BlendMode;

#[test]
fn direct_resident_packets_preserve_full_and_partial_masks() {
    let fresh = packet(17, 4, 4, [0.0, 0.0, 0.0, 1.0], &[work(BlendMode::Replace)], false, true, None)
        .expect("fresh direct packet");
    assert_eq!([4, 24].map(|at| word(&fresh, at)), [10, 15]);
    let replacement = packet(
        18, 4, 4, [0.0, 0.0, 0.0, 1.0], &[work(BlendMode::ReplaceMasked(9))], false, true, Some(17),
    )
    .expect("replacement direct packet");
    assert_eq!([4, 24, 48].map(|at| word(&replacement, at)), [11, 9, 17]);
}

fn work(blend: BlendMode) -> DrawWork {
    DrawWork {
        blend,
        material: DrawMaterial::VertexColor,
        gpu_matrix: None,
        vertices: vec![0; 3 * 32],
        vertex_count: 3,
        viewport: [2.0, 2.0, 0.5, 2.0, 2.0, 0.5],
        scissor: None,
        depth_resource: None,
        depth_state: None,
    }
}

fn word(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().expect("packet word"))
}
