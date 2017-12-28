use game::base::*;

pub fn dist(pos1: &Pos, pos2: &Pos) -> i32 {
    let (x1, y1, z1) = *pos1;
    let (x2, y2, z2) = *pos2;
    let sqr_dist = (x1 - x2).pow(2) + (y1 - y2).pow(2) + (z1 - z2).pow(2);
    (sqr_dist as f64).sqrt() as i32
}
