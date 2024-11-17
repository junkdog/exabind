use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Position};

pub fn blit_buffer(
    src: &Buffer,
    dst: &mut Buffer,
    offset: Offset,
) {
    let mut aux_area = src.area; // guaranteed to be Some
    aux_area.x = offset.x.max(0) as _;
    aux_area.y = offset.y.max(0) as _;

    let target_area = *dst.area();

    let l_clip_x: u16 = offset.x.min(0).unsigned_abs() as _;
    let l_clip_y: u16 = offset.y.min(0).unsigned_abs() as _;

    let r_clip_x: u16 = aux_area.x + aux_area.width - l_clip_x;
    let r_clip_x: u16 = r_clip_x - r_clip_x.min(target_area.width);

    let r_clip_y: u16 = aux_area.y + aux_area.height - l_clip_y;
    let r_clip_y: u16 = r_clip_y - r_clip_y.min(target_area.height);

    if aux_area.width.checked_sub(r_clip_x).is_none()
        || aux_area.height.checked_sub(r_clip_y).is_none()
    {
        return;
    }

    for y in l_clip_y..(aux_area.height - r_clip_y) {
        for x in l_clip_x..(aux_area.width - r_clip_x) {
            if src.cell(Position::new(x, y)).unwrap().skip {
                continue;
            }

            if let (Some(c), Some(new_c)) = (
                dst.cell_mut(Position::new(
                    x + aux_area.x - l_clip_x,
                    y + aux_area.y - l_clip_y,
                )),
                src.cell(Position::new(x, y)),
            ) {
                *c = new_c.clone();
            }
        }
    }
}