#![allow(unused_must_use)]

use core::iter::{range_step};
use qrcode::QrCode;
use std::old_io::{Writer};

const BLACK: &'static str = "\x1B[40m  \x1B[0m";
const WHITE: &'static str = "\x1B[47m  \x1B[0m";

const WW: &'static str = " ";
const WB: &'static str = "▄";
const BW: &'static str = "▀";
const BB: &'static str = "█";

type Bit = bool;
const B: Bit = true;
const W: Bit = false;

// Two-glyph ASCII sequences (0.5 bits/glyph)
pub fn output_ascii<T: Writer>(qr: &QrCode, pad: &str, writer: &mut T) {
    let size = qr.width();
    horiz_border(size, writer, pad, WHITE);
    for y in range(0, size) {
        writer.write_str(pad);
        writer.write_str(WHITE);  // border
        for x in range(0, size) {
            writer.write_str(if qr[(x, y)] { BLACK } else { WHITE });
        }
        writer.write_str(WHITE);  // border
        writer.write_str("\n");
    }
    horiz_border(size, writer, pad, WHITE);
}

// Sub-glyph Unicode sequences (2 bits/glyph)
pub fn output_unicode<T: Writer>(qr: &QrCode, pad: &str, writer: &mut T) {
    let size = qr.width();
    horiz_border(size, writer, pad, BW);
    for y in range_step(0, size, 2) {
        writer.write_str(pad);
        black_on_white(writer);
        writer.write_str(WW);  // border
        for x in range_step(0, size, 1) {
            writer.write_str(glyph(block(qr, x, y)));
        }
        writer.write_str(WW);  // border
        reset_colors(writer);
        writer.write_str("\n");
    }
    if size % 2 == 0 {
        // Due to the implementation of `block`, we get a lower border for free
        // if the QR size is an odd number.
        horiz_border(size, writer, pad, WB);
    }
}

fn black_on_white<T: Writer>(writer: &mut T) {
    writer.write_str("\x1B[30m");  // black foreground
    writer.write_str("\x1B[47m");  // white background
}

fn reset_colors<T: Writer>(writer: &mut T) {
    writer.write_str("\x1B[0m");
}

fn horiz_border<T: Writer>(size: usize, writer: &mut T, pad: &str, white: &str) {
    writer.write_str(pad);
    black_on_white(writer);
    for _ in range(0, size + 2) {
        writer.write_str(white);
    }
    reset_colors(writer);
    writer.write_str("\n");
}

fn glyph(block: (Bit, Bit)) -> &'static str {
    match block {
        (W,W) => WW,
        (W,B) => WB,
        (B,W) => BW,
        (B,B) => BB,
    }
}

// Returns bits from a 1x2 block in the qr code.  The sequence is
// (upper, lower).
fn block(qr: &QrCode, x: usize, y: usize) -> (Bit, Bit) {
    let size = qr.width() - 1;
    (qr[(x, y)], if size > y { qr[(x, y+1)] } else { W })
}

