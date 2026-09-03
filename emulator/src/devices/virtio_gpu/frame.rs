use super::protocol::Rect;
use super::{SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};

const FRAME_MAGIC: &[u8; 4] = b"WBGF";
const FRAME_VERSION: u32 = 1;
const FRAME_HEADER_LEN: usize = 32;

impl VirtioGpu {
    pub(super) fn add_damage(&mut self, resource_id: u32, resource_rect: Rect) {
        let Some(scanout) = self.scanout else {
            return;
        };
        if scanout.resource_id != resource_id {
            return;
        };
        let Some(intersection) = intersect(resource_rect, scanout.rect) else {
            return;
        };
        let damage = Rect {
            x: intersection.x - scanout.rect.x,
            y: intersection.y - scanout.rect.y,
            width: intersection.width,
            height: intersection.height,
        };
        self.pending_damage = Some(self.pending_damage.map_or(damage, |old| union(old, damage)));
    }

    pub(super) fn encode_pending_scanout(&mut self) -> Vec<u8> {
        let Some(damage) = self.pending_damage.take() else {
            return Vec::new();
        };
        let Some(scanout) = self.scanout else {
            return Vec::new();
        };
        let Some(resource) = self.resources.get(&scanout.resource_id) else {
            return Vec::new();
        };
        let Some(payload_len) = (damage.width as usize)
            .checked_mul(damage.height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
        else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload_len);
        out.extend_from_slice(FRAME_MAGIC);
        for value in [
            FRAME_VERSION,
            SCANOUT_WIDTH,
            SCANOUT_HEIGHT,
            damage.x,
            damage.y,
            damage.width,
            damage.height,
        ] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        let source_x = scanout.rect.x + damage.x;
        let source_y = scanout.rect.y + damage.y;
        let row_len = damage.width as usize * 4;
        for row in 0..damage.height {
            let pixel = (source_y + row) as usize * resource.width as usize + source_x as usize;
            let start = pixel * 4;
            out.extend_from_slice(&resource.pixels[start..start + row_len]);
        }
        out
    }
}

fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let end_x = a.x.checked_add(a.width)?.min(b.x.checked_add(b.width)?);
    let end_y = a.y.checked_add(a.height)?.min(b.y.checked_add(b.height)?);
    (end_x > x && end_y > y).then_some(Rect {
        x,
        y,
        width: end_x - x,
        height: end_y - y,
    })
}

fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let end_x = (a.x + a.width).max(b.x + b.width);
    let end_y = (a.y + a.height).max(b.y + b.height);
    Rect {
        x,
        y,
        width: end_x - x,
        height: end_y - y,
    }
}
