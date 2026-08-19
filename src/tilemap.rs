// Ports the RPG Maker 2000/2003 chipset tile-ID -> chipset-pixel mapping,
// including block A/B/D autotile quadrant compositing. The exact block
// ranges and subtile tables are taken from EasyRPG Player's own renderer
// (src/tilemap_layer.cpp, src/map_data.h) since this encoding isn't
// documented anywhere else and getting it wrong produces plausible-looking
// but incorrect tiles. liblcf itself has no rendering code (it's a pure
// data model) so this logic doesn't come from liblcf.
//
// Simplification versus Player: this renders a single static frame (no
// water/animation cycling), since REditor shows a snapshot, not a running
// game.

use image::{Rgba, RgbaImage};

/// Decodes any RPG Maker 2000/2003 graphic (Monster, CharSet, FaceSet, ChipSet, Picture):
/// Handles classic 8-bit indexed PNGs (palette index 0 is transparent) and fallback magenta (#FF00FF) color-keying.
pub fn decode_rpg_image(bytes: &[u8]) -> image::ImageResult<RgbaImage> {
    let mut rgba = image::load_from_memory(bytes)?.to_rgba8();
    let debug = std::env::var("TILEMAP_DEBUG").is_ok();
    let mut keyed = false;

    if let Ok(decoder) = png::Decoder::new(std::io::Cursor::new(bytes)).read_info() {
        let info = decoder.info();
        if info.color_type == png::ColorType::Indexed {
            if let Some(palette) = &info.palette {
                if palette.len() >= 3 {
                    let key = [palette[0], palette[1], palette[2]];
                    for px in rgba.pixels_mut() {
                        if px.0[0] == key[0] && px.0[1] == key[1] && px.0[2] == key[2] {
                            px.0[3] = 0;
                        }
                    }
                    keyed = true;
                    if debug {
                        eprintln!("[decode_rpg_image] keyed palette index 0: {:?}", key);
                    }
                }
            }
        }
    }

    // If not keyed via indexed PNG palette, check if top-left or pure magenta (255, 0, 255) is present as color key
    if !keyed {
        for px in rgba.pixels_mut() {
            if px.0[0] == 255 && px.0[1] == 0 && px.0[2] == 255 {
                px.0[3] = 0;
            }
        }
    }

    Ok(rgba)
}

/// Decodes a chipset PNG using decode_rpg_image.
pub fn decode_chipset(bytes: &[u8]) -> image::ImageResult<RgbaImage> {
    decode_rpg_image(bytes)
}


const TILE: u32 = 16;
const HALF: u32 = TILE / 2;

const BLOCK_A: i32 = 0;
const BLOCK_B_END: i32 = 3000;
const BLOCK_C: i32 = 3000;
const BLOCK_D: i32 = 4000;
const BLOCK_D_END: i32 = 4600;
const BLOCK_E: i32 = 5000;
const BLOCK_F: i32 = 10000;
const BLOCK_F_END: i32 = 10144;

const N: i8 = -1;

// [tile-id][row][col] -> sub-block row within block A's chipset region, or N
// if that quadrant should come from block B instead. Verbatim from Player's
// BlockA_Subtiles_IDS.
#[rustfmt::skip]
const BLOCK_A_SUBTILES: [[[i8; 2]; 2]; 47] = [
    [[N, N], [N, N]],
    [[3, N], [N, N]],
    [[N, 3], [N, N]],
    [[3, 3], [N, N]],
    [[N, N], [N, 3]],
    [[3, N], [N, 3]],
    [[N, 3], [N, 3]],
    [[3, 3], [N, 3]],
    [[N, N], [3, N]],
    [[3, N], [3, N]],
    [[N, 3], [3, N]],
    [[3, 3], [3, N]],
    [[N, N], [3, 3]],
    [[3, N], [3, 3]],
    [[N, 3], [3, 3]],
    [[3, 3], [3, 3]],
    [[1, N], [1, N]],
    [[1, 3], [1, N]],
    [[1, N], [1, 3]],
    [[1, 3], [1, 3]],
    [[2, 2], [N, N]],
    [[2, 2], [N, 3]],
    [[2, 2], [3, N]],
    [[2, 2], [3, 3]],
    [[N, 1], [N, 1]],
    [[N, 1], [3, 1]],
    [[3, 1], [N, 1]],
    [[3, 1], [3, 1]],
    [[N, N], [2, 2]],
    [[3, N], [2, 2]],
    [[N, 3], [2, 2]],
    [[3, 3], [2, 2]],
    [[1, 1], [1, 1]],
    [[2, 2], [2, 2]],
    [[0, 2], [1, N]],
    [[0, 2], [1, 3]],
    [[2, 0], [N, 1]],
    [[2, 0], [3, 1]],
    [[N, 1], [2, 0]],
    [[3, 1], [2, 0]],
    [[1, N], [0, 2]],
    [[1, 3], [0, 2]],
    [[0, 0], [1, 1]],
    [[0, 2], [0, 2]],
    [[1, 1], [0, 0]],
    [[2, 0], [2, 0]],
    [[0, 0], [0, 0]],
];

// [tile-id][row][col][x/y] chipset-tile-unit offsets within a D block.
// Verbatim from Player's BlockD_Subtiles_IDS.
#[rustfmt::skip]
const BLOCK_D_SUBTILES: [[[[u8; 2]; 2]; 2]; 50] = [
    [[[1, 2], [1, 2]], [[1, 2], [1, 2]]],
    [[[2, 0], [1, 2]], [[1, 2], [1, 2]]],
    [[[1, 2], [2, 0]], [[1, 2], [1, 2]]],
    [[[2, 0], [2, 0]], [[1, 2], [1, 2]]],
    [[[1, 2], [1, 2]], [[1, 2], [2, 0]]],
    [[[2, 0], [1, 2]], [[1, 2], [2, 0]]],
    [[[1, 2], [2, 0]], [[1, 2], [2, 0]]],
    [[[2, 0], [2, 0]], [[1, 2], [2, 0]]],
    [[[1, 2], [1, 2]], [[2, 0], [1, 2]]],
    [[[2, 0], [1, 2]], [[2, 0], [1, 2]]],
    [[[1, 2], [2, 0]], [[2, 0], [1, 2]]],
    [[[2, 0], [2, 0]], [[2, 0], [1, 2]]],
    [[[1, 2], [1, 2]], [[2, 0], [2, 0]]],
    [[[2, 0], [1, 2]], [[2, 0], [2, 0]]],
    [[[1, 2], [2, 0]], [[2, 0], [2, 0]]],
    [[[2, 0], [2, 0]], [[2, 0], [2, 0]]],
    [[[0, 2], [0, 2]], [[0, 2], [0, 2]]],
    [[[0, 2], [2, 0]], [[0, 2], [0, 2]]],
    [[[0, 2], [0, 2]], [[0, 2], [2, 0]]],
    [[[0, 2], [2, 0]], [[0, 2], [2, 0]]],
    [[[1, 1], [1, 1]], [[1, 1], [1, 1]]],
    [[[1, 1], [1, 1]], [[1, 1], [2, 0]]],
    [[[1, 1], [1, 1]], [[2, 0], [1, 1]]],
    [[[1, 1], [1, 1]], [[2, 0], [2, 0]]],
    [[[2, 2], [2, 2]], [[2, 2], [2, 2]]],
    [[[2, 2], [2, 2]], [[2, 0], [2, 2]]],
    [[[2, 0], [2, 2]], [[2, 2], [2, 2]]],
    [[[2, 0], [2, 2]], [[2, 0], [2, 2]]],
    [[[1, 3], [1, 3]], [[1, 3], [1, 3]]],
    [[[2, 0], [1, 3]], [[1, 3], [1, 3]]],
    [[[1, 3], [2, 0]], [[1, 3], [1, 3]]],
    [[[2, 0], [2, 0]], [[1, 3], [1, 3]]],
    [[[0, 2], [2, 2]], [[0, 2], [2, 2]]],
    [[[1, 1], [1, 1]], [[1, 3], [1, 3]]],
    [[[0, 1], [0, 1]], [[0, 1], [0, 1]]],
    [[[0, 1], [0, 1]], [[0, 1], [2, 0]]],
    [[[2, 1], [2, 1]], [[2, 1], [2, 1]]],
    [[[2, 1], [2, 1]], [[2, 0], [2, 1]]],
    [[[2, 3], [2, 3]], [[2, 3], [2, 3]]],
    [[[2, 0], [2, 3]], [[2, 3], [2, 3]]],
    [[[0, 3], [0, 3]], [[0, 3], [0, 3]]],
    [[[0, 3], [2, 0]], [[0, 3], [0, 3]]],
    [[[0, 1], [2, 1]], [[0, 1], [2, 1]]],
    [[[0, 1], [0, 1]], [[0, 3], [0, 3]]],
    [[[0, 3], [2, 3]], [[0, 3], [2, 3]]],
    [[[2, 1], [2, 1]], [[2, 3], [2, 3]]],
    [[[0, 1], [2, 1]], [[0, 3], [2, 3]]],
    [[[1, 2], [1, 2]], [[1, 2], [1, 2]]],
    [[[1, 2], [1, 2]], [[1, 2], [1, 2]]],
    [[[0, 0], [0, 0]], [[0, 0], [0, 0]]],
];

/// Copies an 8x8 (HALF x HALF) block from `src` to `dst`. Silently clamps/
/// skips if either side is out of bounds, rather than panicking on a
/// malformed or unusually small chipset image.
fn blit_half(dst: &mut RgbaImage, dst_x: u32, dst_y: u32, src: &RgbaImage, src_x: u32, src_y: u32) {
    for dy in 0..HALF {
        for dx in 0..HALF {
            let (sx, sy) = (src_x + dx, src_y + dy);
            let (tx, ty) = (dst_x + dx, dst_y + dy);
            if sx < src.width() && sy < src.height() && tx < dst.width() && ty < dst.height() {
                let px: Rgba<u8> = *src.get_pixel(sx, sy);
                if px.0[3] != 0 {
                    dst.put_pixel(tx, ty, px);
                }
            }
        }
    }
}

fn blit_tile(dst: &mut RgbaImage, dst_x: u32, dst_y: u32, src: &RgbaImage, src_col: i32, src_row: i32) {
    if src_col < 0 || src_row < 0 {
        return;
    }
    let (src_x, src_y) = (src_col as u32 * TILE, src_row as u32 * TILE);
    for dy in 0..TILE {
        for dx in 0..TILE {
            let (sx, sy) = (src_x + dx, src_y + dy);
            let (tx, ty) = (dst_x + dx, dst_y + dy);
            if sx < src.width() && sy < src.height() && tx < dst.width() && ty < dst.height() {
                let px: Rgba<u8> = *src.get_pixel(sx, sy);
                if px.0[3] != 0 {
                    dst.put_pixel(tx, ty, px);
                }
            }
        }
    }
}

/// Quarters as (col, row) chipset *tile* coordinates (16px units); a quarter
/// picks one 8x8 corner of that tile when blitted (see `blit_half` callers).
type Quarters = [[(i32, i32); 2]; 2];

fn autotile_ab_quarters(id: i32) -> Quarters {
    // block: 0 = A1, 1 = A2, 2 = B (matches BLOCK_A_END=2000, BLOCK_B_END=3000)
    let block = id / 1000;
    let b_subtile = (id - block * 1000) / 50;
    let a_subtile = (id - block * 1000 - b_subtile * 50) as usize;

    let mut quarters: Quarters = [[(0, 0); 2]; 2];
    let a_subtile = a_subtile.min(BLOCK_A_SUBTILES.len() - 1);

    // Pass 1: fill quadrants not covered by block A's corner shapes with block B.
    for j in 0..2 {
        for i in 0..2 {
            if BLOCK_A_SUBTILES[a_subtile][j][i] == N {
                let mut t = (b_subtile >> (j * 2 + i)) & 1;
                if block == 2 {
                    t ^= 3;
                }
                // animID fixed at 0 (static render, no water-animation cycling)
                quarters[j][i] = (0, 4 + t);
            }
        }
    }
    // Pass 2: fill quadrants covered by block A's corner shapes.
    for j in 0..2 {
        for i in 0..2 {
            let a = BLOCK_A_SUBTILES[a_subtile][j][i];
            if a != N {
                let col = if block == 1 { 3 } else { 0 };
                quarters[j][i] = (col, a as i32);
            }
        }
    }
    // Pass 3: when combining an edge (b_subtile) with a corner (a_subtile),
    // some quadrants get overridden back to block B.
    if b_subtile != 0 && a_subtile != 0 {
        for j in 0..2 {
            for i in 0..2 {
                let mut t = (b_subtile >> (j * 2 + i)) & 1;
                if block == 2 {
                    t *= 2;
                }
                if t == 0 {
                    continue;
                }
                quarters[j][i] = (0, 4 + t);
            }
        }
    }
    quarters
}

fn autotile_d_quarters(id: i32) -> Quarters {
    let rel = id - BLOCK_D;
    let block = (rel / 50).clamp(0, 11) as usize;
    let subtile = ((rel - block as i32 * 50) as usize).min(BLOCK_D_SUBTILES.len() - 1);

    let (block_x, block_y) = if block < 4 {
        ((block % 2) as i32 * 3, 8 + (block / 2) as i32 * 4)
    } else {
        (6 + (block % 2) as i32 * 3, ((block - 4) / 2) as i32 * 4)
    };

    let mut quarters: Quarters = [[(0, 0); 2]; 2];
    for j in 0..2 {
        for i in 0..2 {
            let [ox, oy] = BLOCK_D_SUBTILES[subtile][j][i];
            quarters[j][i] = (block_x + ox as i32, block_y + oy as i32);
        }
    }
    quarters
}

fn draw_quarters(dst: &mut RgbaImage, dst_x: u32, dst_y: u32, chipset: &RgbaImage, quarters: &Quarters) {
    for j in 0..2u32 {
        for i in 0..2u32 {
            let (col, row) = quarters[j as usize][i as usize];
            if col < 0 || row < 0 {
                continue;
            }
            let src_x = col as u32 * TILE + i * HALF;
            let src_y = row as u32 * TILE + j * HALF;
            blit_half(dst, dst_x + i * HALF, dst_y + j * HALF, chipset, src_x, src_y);
        }
    }
}

/// Draws one map tile (by its raw liblcf tile ID) at the given destination
/// pixel position. `upper` selects block F (upper layer) vs. the lower-layer
/// blocks (A/B/C/D/E). Unrecognized IDs are left blank rather than guessed.
fn draw_tile(dst: &mut RgbaImage, dst_x: u32, dst_y: u32, chipset: &RgbaImage, id: i32, upper: bool) {
    if upper {
        if id >= BLOCK_F && id < BLOCK_F_END {
            let local = id - BLOCK_F;
            let (col, row) = if local < 48 {
                (18 + local % 6, 8 + local / 6)
            } else {
                (24 + (local - 48) % 6, (local - 48) / 6)
            };
            blit_tile(dst, dst_x, dst_y, chipset, col, row);
        }
        return;
    }

    if id >= BLOCK_E {
        let local = id - BLOCK_E;
        let (col, row) = if local < 96 {
            (12 + local % 6, local / 6)
        } else {
            (18 + (local - 96) % 6, (local - 96) / 6)
        };
        blit_tile(dst, dst_x, dst_y, chipset, col, row);
    } else if id >= BLOCK_D && id < BLOCK_D_END {
        let quarters = autotile_d_quarters(id);
        draw_quarters(dst, dst_x, dst_y, chipset, &quarters);
    } else if id >= BLOCK_C && id < BLOCK_D {
        let col = 3 + (id - BLOCK_C) / 50;
        let row = 4; // static frame (no water/waterfall animation)
        blit_tile(dst, dst_x, dst_y, chipset, col, row);
    } else if id >= BLOCK_A && id < BLOCK_B_END {
        let quarters = autotile_ab_quarters(id);
        draw_quarters(dst, dst_x, dst_y, chipset, &quarters);
    }
    // else: unrecognized ID -> leave transparent rather than guess.
}

/// Whether `draw_tile` would actually draw something for this ID (vs.
/// silently leaving it blank because it falls outside every known block
/// range). Useful for scanning real project data for format gaps.
pub fn is_recognized(id: i32, upper: bool) -> bool {
    if upper {
        return id >= BLOCK_F && id < BLOCK_F_END;
    }
    id >= BLOCK_E
        || (id >= BLOCK_D && id < BLOCK_D_END)
        || (id >= BLOCK_C && id < BLOCK_D)
        || (id >= BLOCK_A && id < BLOCK_B_END)
}

// Chipset passability byte bits (from EasyRPG Player's Passable enum).
const PASSABLE_DIRECTIONS: u8 = 0x01 | 0x02 | 0x04 | 0x08; // Down|Left|Right|Up

/// Maps a lower-layer tile ID to its index into `Chipset.passable_data_lower`
/// (162 entries: A1/A2=0-1, B=2, C=3-5, D=6-17, E=18-161). Verbatim from
/// Player's `ChipIdToIndex` in src/map_data.h.
fn passability_index_lower(id: i32) -> Option<usize> {
    if id >= BLOCK_A && id < 2000 {
        Some((0 + (id - BLOCK_A) / 1000) as usize)
    } else if (2000..BLOCK_B_END).contains(&id) {
        Some((2 + (id - 2000) / 1000) as usize)
    } else if id >= BLOCK_C && id < BLOCK_D {
        Some((3 + (id - BLOCK_C) / 50) as usize)
    } else if id >= BLOCK_D && id < BLOCK_D_END {
        Some((6 + (id - BLOCK_D) / 50) as usize)
    } else if id >= BLOCK_E {
        Some((18 + (id - BLOCK_E)) as usize)
    } else {
        None
    }
}

/// Maps an upper-layer (block F) tile ID to its index into
/// `Chipset.passable_data_upper` (144 entries, 0-based within the block -
/// unlike the lower blocks, this array isn't part of the shared 0-161
/// indexing scheme, see NUM_UPPER_TILES in Player's map_data.h).
fn passability_index_upper(id: i32) -> Option<usize> {
    if id >= BLOCK_F && id < BLOCK_F_END {
        Some((id - BLOCK_F) as usize)
    } else {
        None
    }
}

/// Whether a map cell is impassable, combining both layers. Simplification:
/// this checks "can you exit in *some* direction from this tile" (all of
/// Down/Left/Right/Up blocked on either layer), not RPG Maker's full
/// pairwise tile-to-tile movement rules (which also consult the
/// *destination* tile's opposite-direction bit) - good enough for a level-
/// design overlay, not a movement simulator. Missing passability data
/// (index out of range) fails open (treated as passable) rather than
/// falsely flagging tiles as blocked.
pub fn is_blocked(lower_id: i32, upper_id: i32, passable_lower: &[u8], passable_upper: &[u8]) -> bool {
    let lower_bits = passability_index_lower(lower_id)
        .and_then(|i| passable_lower.get(i))
        .copied()
        .unwrap_or(PASSABLE_DIRECTIONS)
        & PASSABLE_DIRECTIONS;
    let upper_bits = passability_index_upper(upper_id)
        .and_then(|i| passable_upper.get(i))
        .copied()
        .unwrap_or(PASSABLE_DIRECTIONS)
        & PASSABLE_DIRECTIONS;
    (lower_bits & upper_bits) == 0
}

/// Composites a full map from its lower/upper tile-ID layers and the
/// decoded chipset image. Returns an image sized `width*16 x height*16`.
pub fn render_map(chipset: &RgbaImage, width: i32, height: i32, lower: &[i32], upper: &[i32]) -> RgbaImage {
    let (w, h) = (width.max(0) as u32, height.max(0) as u32);
    let mut out = RgbaImage::from_pixel(w * TILE, h * TILE, Rgba([0, 0, 0, 0]));

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let (dst_x, dst_y) = (x * TILE, y * TILE);

            if let Some(&id) = lower.get(idx) {
                draw_tile(&mut out, dst_x, dst_y, chipset, id, false);
            }
            if let Some(&id) = upper.get(idx) {
                draw_tile(&mut out, dst_x, dst_y, chipset, id, true);
            }
        }
    }

    out
}

/// Computes the autotile subtile index (0..46) for Block D from its 8 neighbors.
pub fn calculate_autotile_d_subtile(
    n: bool,
    ne: bool,
    e: bool,
    se: bool,
    s: bool,
    sw: bool,
    w: bool,
    nw: bool,
) -> usize {
    // Quadrant logic: [j][i]
    // Top-Left (j=0, i=0): N, W, NW
    let tl = if !n && !w {
        [1, 2] // corner
    } else if n && !w {
        [0, 2] // vert edge
    } else if !n && w {
        [1, 1] // horiz edge
    } else if !nw {
        [2, 0] // inner corner
    } else {
        [0, 1] // center
    };

    // Top-Right (j=0, i=1): N, E, NE
    let tr = if !n && !e {
        [1, 2]
    } else if n && !e {
        [0, 2]
    } else if !n && e {
        [1, 1]
    } else if !ne {
        [2, 0]
    } else {
        [2, 1]
    };

    // Bottom-Left (j=1, i=0): S, W, SW
    let bl = if !s && !w {
        [1, 2]
    } else if s && !w {
        [0, 2]
    } else if !s && w {
        [1, 1]
    } else if !sw {
        [2, 0]
    } else {
        [0, 3]
    };

    // Bottom-Right (j=1, i=1): S, E, SE
    let br = if !s && !e {
        [1, 2]
    } else if s && !e {
        [0, 2]
    } else if !s && e {
        [1, 1]
    } else if !se {
        [2, 0]
    } else {
        [2, 3]
    };

    // Find the subtile pattern in BLOCK_D_SUBTILES
    for (idx, subtile) in BLOCK_D_SUBTILES.iter().enumerate() {
        if subtile[0][0] == tl && subtile[0][1] == tr && subtile[1][0] == bl && subtile[1][1] == br {
            return idx;
        }
    }
    0
}

/// Check if a tile ID belongs to the same autotile block (e.g. D0..D11).
fn autotile_block_base(id: i32) -> Option<i32> {
    if (BLOCK_D..BLOCK_D_END).contains(&id) {
        let block_idx = (id - BLOCK_D) / 50;
        Some(BLOCK_D + block_idx * 50)
    } else {
        None
    }
}

pub fn is_autotile_d(id: i32) -> bool {
    autotile_block_base(id).is_some()
}

/// Updates the subtile calculation for a specific cell on the lower layer if it's an autotile.
pub fn update_autotile_at(lower: &mut [i32], width: i32, height: i32, x: i32, y: i32) {
    if x < 0 || x >= width || y < 0 || y >= height {
        return;
    }
    let idx = (y * width + x) as usize;
    let tile_id = match lower.get(idx) {
        Some(&t) => t,
        None => return,
    };

    if let Some(base) = autotile_block_base(tile_id) {
        let same = |nx: i32, ny: i32| -> bool {
            if nx < 0 || nx >= width || ny < 0 || ny >= height {
                return true; // wrap/border behaves as connected
            }
            let n_idx = (ny * width + nx) as usize;
            lower.get(n_idx).and_then(|&id| autotile_block_base(id)) == Some(base)
        };

        let n = same(x, y - 1);
        let s = same(x, y + 1);
        let w = same(x - 1, y);
        let e = same(x + 1, y);
        let nw = same(x - 1, y - 1);
        let ne = same(x + 1, y - 1);
        let sw = same(x - 1, y + 1);
        let se = same(x + 1, y + 1);

        let subtile = calculate_autotile_d_subtile(n, ne, e, se, s, sw, w, nw);
        lower[idx] = base + subtile as i32;
    }
}

/// Updates a tile and its 8 adjacent neighbors if they are autotiles.
pub fn update_autotile_neighborhood(lower: &mut [i32], width: i32, height: i32, cx: i32, cy: i32) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            update_autotile_at(lower, width, height, cx + dx, cy + dy);
        }
    }
}

pub const PALETTE_COLS: usize = 6;

/// Generates a palette image and a mapping of `(row * PALETTE_COLS + col) -> tile_id`.
pub fn render_palette_image(chipset: &RgbaImage, is_upper: bool) -> (RgbaImage, Vec<i32>) {
    let mut tile_ids = Vec::new();

    if is_upper {
        // 144 upper tiles (6 cols x 24 rows)
        // tile IDs: 10000..10143
        for id in BLOCK_F..BLOCK_F_END {
            tile_ids.push(id);
        }
    } else {
        // Row 0: Special autotiles A1 (0), A2 (1000), B (2000), C1 (3000), C2 (3050), E0 (5000)
        tile_ids.push(0);
        tile_ids.push(1000);
        tile_ids.push(2000);
        tile_ids.push(3000);
        tile_ids.push(3050);
        tile_ids.push(5000);

        // Row 1: Land autotiles D0..D5 (IDs 4000, 4050, 4100, 4150, 4200, 4250)
        for i in 0..6 {
            tile_ids.push(BLOCK_D + i * 50);
        }
        // Row 2: Land autotiles D6..D11 (IDs 4300, 4350, 4400, 4450, 4500, 4550)
        for i in 6..12 {
            tile_ids.push(BLOCK_D + i * 50);
        }
        // Rows 3..26: Block E regular tiles (144 tiles = IDs 5000..5143)
        for id in BLOCK_E..BLOCK_E + 144 {
            tile_ids.push(id);
        }
    }

    let rows = (tile_ids.len() + PALETTE_COLS - 1) / PALETTE_COLS;
    let (pw, ph) = (PALETTE_COLS as u32 * TILE, rows as u32 * TILE);
    let mut out = RgbaImage::from_pixel(pw, ph, Rgba([30, 30, 30, 255]));

    for (idx, &id) in tile_ids.iter().enumerate() {
        let col = (idx % PALETTE_COLS) as u32;
        let row = (idx / PALETTE_COLS) as u32;
        let (dst_x, dst_y) = (col * TILE, row * TILE);

        // Checkerboard background for transparency
        for dy in 0..TILE {
            for dx in 0..TILE {
                let dark = ((dx / 4) + (dy / 4)) % 2 == 0;
                let c = if dark { 45 } else { 55 };
                out.put_pixel(dst_x + dx, dst_y + dy, Rgba([c, c, c, 255]));
            }
        }

        draw_tile(&mut out, dst_x, dst_y, chipset, id, is_upper);
    }

    (out, tile_ids)
}

