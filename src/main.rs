/*
 * srx: The fast Symbol Ranking based compressor.
 * Copyright (C) 2023  Mai Thanh Minh (a.k.a. thanhminhmr)
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either  version 3 of the  License,  or (at your option) any later
 * version.
 *
 * This program  is distributed in the hope  that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR  A PARTICULAR PURPOSE. See  the  GNU  General  Public   License  for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Seek, Write};
use std::path::Path;
use std::process::exit;
use std::time::Instant;

// =================================================================================================

const MULTIPLIER: [u32; 256] = [
	0x80000000, 0x55555555, 0x40000000, 0x33333333, 0x2AAAAAAA, 0x24924924, 0x20000000, 0x1C71C71C,
	0x19999999, 0x1745D174, 0x15555555, 0x13B13B13, 0x12492492, 0x11111111, 0x10000000, 0x0F0F0F0F,
	0x0E38E38E, 0x0D79435E, 0x0CCCCCCC, 0x0C30C30C, 0x0BA2E8BA, 0x0B21642C, 0x0AAAAAAA, 0x0A3D70A3,
	0x09D89D89, 0x097B425E, 0x09249249, 0x08D3DCB0, 0x08888888, 0x08421084, 0x08000000, 0x07C1F07C,
	0x07878787, 0x07507507, 0x071C71C7, 0x06EB3E45, 0x06BCA1AF, 0x06906906, 0x06666666, 0x063E7063,
	0x06186186, 0x05F417D0, 0x05D1745D, 0x05B05B05, 0x0590B216, 0x0572620A, 0x05555555, 0x05397829,
	0x051EB851, 0x05050505, 0x04EC4EC4, 0x04D4873E, 0x04BDA12F, 0x04A7904A, 0x04924924, 0x047DC11F,
	0x0469EE58, 0x0456C797, 0x04444444, 0x04325C53, 0x04210842, 0x04104104, 0x04000000, 0x03F03F03,
	0x03E0F83E, 0x03D22635, 0x03C3C3C3, 0x03B5CC0E, 0x03A83A83, 0x039B0AD1, 0x038E38E3, 0x0381C0E0,
	0x03759F22, 0x0369D036, 0x035E50D7, 0x03531DEC, 0x03483483, 0x033D91D2, 0x03333333, 0x0329161F,
	0x031F3831, 0x03159721, 0x030C30C3, 0x03030303, 0x02FA0BE8, 0x02F14990, 0x02E8BA2E, 0x02E05C0B,
	0x02D82D82, 0x02D02D02, 0x02C8590B, 0x02C0B02C, 0x02B93105, 0x02B1DA46, 0x02AAAAAA, 0x02A3A0FD,
	0x029CBC14, 0x0295FAD4, 0x028F5C28, 0x0288DF0C, 0x02828282, 0x027C4597, 0x02762762, 0x02702702,
	0x026A439F, 0x02647C69, 0x025ED097, 0x02593F69, 0x0253C825, 0x024E6A17, 0x02492492, 0x0243F6F0,
	0x023EE08F, 0x0239E0D5, 0x0234F72C, 0x02302302, 0x022B63CB, 0x0226B902, 0x02222222, 0x021D9EAD,
	0x02192E29, 0x0214D021, 0x02108421, 0x020C49BA, 0x02082082, 0x02040810, 0x02000000, 0x01FC07F0,
	0x01F81F81, 0x01F44659, 0x01F07C1F, 0x01ECC07B, 0x01E9131A, 0x01E573AC, 0x01E1E1E1, 0x01DE5D6E,
	0x01DAE607, 0x01D77B65, 0x01D41D41, 0x01D0CB58, 0x01CD8568, 0x01CA4B30, 0x01C71C71, 0x01C3F8F0,
	0x01C0E070, 0x01BDD2B8, 0x01BACF91, 0x01B7D6C3, 0x01B4E81B, 0x01B20364, 0x01AF286B, 0x01AC5701,
	0x01A98EF6, 0x01A6D01A, 0x01A41A41, 0x01A16D3F, 0x019EC8E9, 0x019C2D14, 0x01999999, 0x01970E4F,
	0x01948B0F, 0x01920FB4, 0x018F9C18, 0x018D3018, 0x018ACB90, 0x01886E5F, 0x01861861, 0x0183C977,
	0x01818181, 0x017F405F, 0x017D05F4, 0x017AD220, 0x0178A4C8, 0x01767DCE, 0x01745D17, 0x01724287,
	0x01702E05, 0x016E1F76, 0x016C16C1, 0x016A13CD, 0x01681681, 0x01661EC6, 0x01642C85, 0x01623FA7,
	0x01605816, 0x015E75BB, 0x015C9882, 0x015AC056, 0x0158ED23, 0x01571ED3, 0x01555555, 0x01539094,
	0x0151D07E, 0x01501501, 0x014E5E0A, 0x014CAB88, 0x014AFD6A, 0x0149539E, 0x0147AE14, 0x01460CBC,
	0x01446F86, 0x0142D662, 0x01414141, 0x013FB013, 0x013E22CB, 0x013C995A, 0x013B13B1, 0x013991C2,
	0x01381381, 0x013698DF, 0x013521CF, 0x0133AE45, 0x01323E34, 0x0130D190, 0x012F684B, 0x012E025C,
	0x012C9FB4, 0x012B404A, 0x0129E412, 0x01288B01, 0x0127350B, 0x0125E227, 0x01249249, 0x01234567,
	0x0121FB78, 0x0120B470, 0x011F7047, 0x011E2EF3, 0x011CF06A, 0x011BB4A4, 0x011A7B96, 0x01194538,
	0x01181181, 0x0116E068, 0x0115B1E5, 0x011485F0, 0x01135C81, 0x0112358E, 0x01111111, 0x010FEF01,
	0x010ECF56, 0x010DB20A, 0x010C9714, 0x010B7E6E, 0x010A6810, 0x010953F3, 0x01084210, 0x01073260,
	0x010624DD, 0x0105197F, 0x01041041, 0x0103091B, 0x01020408, 0x01010101, 0x01000000, 0x00FF00FF,
];

#[derive(Clone)]
struct BitPrediction {
	state: u32, // lower 8-bit is a counter, higher 24-bit is prediction
}

impl BitPrediction {
	fn new() -> Self {
		Self {
			state: 0x80000000
		}
	}

	fn get_prediction(&self) -> u64 { (self.state >> 8) as u64 }

	fn update(&mut self, bit: usize) -> u64 {
		assert!(bit == 0 || bit == 1);
		// get bit 0-7 as count
		let count: usize = (self.state & 0xFF) as usize;
		// get bit 8-31 as current prediction
		let current_prediction: i64 = (self.state >> 8) as i64;
		// create bit shift
		let bit_shift: i64 = (bit as i64) << 24;
		// get multiplier
		let multiplier: i64 = MULTIPLIER[count] as i64;
		// calculate new prediction
		let new_prediction: u32 = (((bit_shift - current_prediction) * multiplier) >> 24) as u32;
		// update state
		self.state = self.state.wrapping_add((new_prediction & 0xFFFFFF00)
			+ if count < 255 { 1 } else { 0 });
		// return current prediction (before update)
		return current_prediction as u64;
	}
}

// -----------------------------------------------

struct BitEncoder<'a, W: Write> {
	low: u64,
	high: u64,
	states: Vec<BitPrediction>,
	bit_buffer: u8,
	bit_count: usize,
	output: &'a mut W,
}

impl<'a, W: Write> BitEncoder<'a, W> {
	fn new(size: usize, output: &'a mut W) -> Self {
		Self {
			low: 0,
			high: 0xFFFFFFFF_FFFFFFFF,
			states: vec![BitPrediction::new(); size],
			bit_buffer: 0,
			bit_count: 0,
			output,
		}
	}

	fn bit(&mut self, context: usize, bit: usize) -> Result<()> {
		assert!(bit == 0 || bit == 1);
		// get prediction
		let prediction: u64 = self.states[context].update(bit);
		// get delta
		assert!(self.low < self.high);
		let delta: u64 = (((self.high - self.low) as u128 * prediction as u128) >> 24) as u64;
		// calculate middle
		let mid: u64 = self.low + delta + (bit ^ 1) as u64;
		assert!(mid >= self.low && mid < self.high);
		// set new range limit
		*(if bit != 0 { &mut self.high } else { &mut self.low }) = mid;
		// shift bits out
		return self.flush_bit();
	}

	fn byte(&mut self, context: usize, byte: u8) -> Result<()> {
		// code high 4 bits in first 15 contexts
		let high: usize = ((byte >> 4) | 16) as usize;
		self.bit(context + 1, high >> 3 & 1)?;
		self.bit(context + (high >> 3), high >> 2 & 1)?;
		self.bit(context + (high >> 2), high >> 1 & 1)?;
		self.bit(context + (high >> 1), high & 1)?;
		// code low 4 bits in one of 16 blocks of 15 contexts (to reduce cache misses)
		let low_context: usize = context + (15 * (high - 15)) as usize;
		let low: usize = ((byte & 15) | 16) as usize;
		self.bit(low_context + 1, low >> 3 & 1)?;
		self.bit(low_context + (low >> 3), low >> 2 & 1)?;
		self.bit(low_context + (low >> 2), low >> 1 & 1)?;
		self.bit(low_context + (low >> 1), low & 1)?;
		// oke
		return Ok(());
	}

	fn flush_bit(&mut self) -> Result<()> {
		// shift bits out
		while (self.high ^ self.low) & 0x80000000_00000000 == 0 {
			// add to bit buffer
			self.bit_buffer += self.bit_buffer
				+ if self.low >= 0x80000000_00000000 { 1 } else { 0 };
			// shift new bit into high/low
			self.high += self.high + 1;
			self.low += self.low;
			// flush
			self.bit_count += 1;
			if self.bit_count == 8 {
				self.output.write_all(&[self.bit_buffer])?;
				self.bit_count = 0;
			}
		}
		// oke
		return Ok(());
	}

	fn flush(&mut self) -> Result<()> {
		// add last bit from low
		self.bit_buffer += self.bit_buffer
			+ if self.low >= 0x80000000_00000000 { 1 } else { 0 };
		self.bit_count += 1;
		self.bit_buffer <<= 8 - self.bit_count;
		// write then get out
		return self.output.write_all(&[self.bit_buffer]);
	}
}

// =================================================================================================

enum ByteMatched {
	FIRST,
	SECOND,
	THIRD,
	NONE,
}

#[derive(Clone)]
struct MatchingContext {
	value: u32,
}

impl MatchingContext {
	fn new() -> Self {
		Self {
			value: 0
		}
	}

	fn get_first(&self) -> u8 { self.value as u8 }
	fn get_second(&self) -> u8 { (self.value >> 8) as u8 }
	fn get_third(&self) -> u8 { (self.value >> 16) as u8 }
	fn get_count(&self) -> usize { (self.value >> 24) as usize }

	fn update_match(&mut self, next_byte: u8) -> ByteMatched {
		let mask: u32 = self.value ^ (0x10101 * next_byte as u32);
		return if (mask & 0x0000FF) == 0 { // mask for the first byte
			// increase count by 1, capped at 255
			self.value += if self.value < 0xFF000000 { 0x01000000 } else { 0 };

			ByteMatched::FIRST
		} else if (mask & 0x00FF00) == 0 { // mask for the second byte
			self.value = (self.value & 0xFF0000) // keep the third byte
				| ((self.value << 8) & 0xFF00) // bring the old first byte to second place
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

			ByteMatched::SECOND
		} else if (mask & 0xFF0000) == 0 {  // mask for the third byte
			self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

			ByteMatched::THIRD
		} else { // not match
			self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32; // set the first byte

			ByteMatched::NONE
		};
	}
}

// -----------------------------------------------

struct MatchingContexts {
	last_byte: u8,
	hash_value: usize,
	contexts: Vec<MatchingContext>,
}

impl MatchingContexts {
	fn new(size_log: usize) -> Self {
		Self {
			last_byte: 0,
			hash_value: 0,
			contexts: vec![MatchingContext::new(); 1 << size_log],
		}
	}

	fn get_last_byte(&self) -> u8 { self.last_byte }
	fn get_hash_value(&self) -> usize { self.hash_value }
	fn get_context(&self) -> &MatchingContext { &self.contexts[self.hash_value] }

	fn update_match(&mut self, next_byte: u8) -> ByteMatched {
		let matching_byte: ByteMatched = self.contexts[self.hash_value].update_match(next_byte);
		self.last_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + next_byte as usize + 1)
			& (self.contexts.len() - 1);
		return matching_byte;
	}
}

// =================================================================================================

struct StreamEncoder<'a, R: Read, W: Write> {
	contexts: MatchingContexts,
	encoder: BitEncoder<'a, W>,
	input: &'a mut R,
}

impl<'a, R: Read, W: Write> StreamEncoder<'a, R, W> {
	fn new(input: &'a mut R, output: &'a mut W, context_size_log: usize) -> Self <> {
		Self {
			contexts: MatchingContexts::new(context_size_log),
			encoder: BitEncoder::new((1024 + 256) * 1024, output),
			input,
		}
	}

	fn encode(&mut self) -> Result<()> {
		let contexts: &mut MatchingContexts = &mut self.contexts;
		let encoder: &mut BitEncoder<W> = &mut self.encoder;
		loop {
			let matching_context: &MatchingContext = contexts.get_context();
			let count: usize = matching_context.get_count();

			let bit_context: usize = if count < 4 {
				((contexts.get_last_byte() as usize) << 2) | count
			} else {
				1024 + count
			} * 1024;

			let first_byte: u8 = matching_context.get_first();
			let second_byte: u8 = matching_context.get_second();
			let third_byte: u8 = matching_context.get_third();

			let first_context: usize = bit_context + first_byte as usize;
			let second_context: usize = bit_context + 256
				+ second_byte.wrapping_add(third_byte) as usize;
			let third_context: usize = bit_context + 512
				+ second_byte.wrapping_mul(2).wrapping_sub(third_byte) as usize;
			let literal_context: usize = bit_context + 768;

			let mut byte_result: [u8; 1] = [0];
			if self.input.read(&mut byte_result)? == 0 {
				encoder.bit(first_context, 1)?;
				encoder.bit(second_context, 0)?;
				encoder.byte(literal_context, first_byte)?;
				return encoder.flush();
			}

			let current_byte: u8 = byte_result[0];

			match contexts.update_match(current_byte) {
				ByteMatched::FIRST => {
					encoder.bit(first_context, 0)?;
				}
				ByteMatched::SECOND => {
					encoder.bit(first_context, 1)?;
					encoder.bit(second_context, 1)?;
					encoder.bit(third_context, 0)?;
				}
				ByteMatched::THIRD => {
					encoder.bit(first_context, 1)?;
					encoder.bit(second_context, 1)?;
					encoder.bit(third_context, 1)?;
				}
				ByteMatched::NONE => {
					encoder.bit(first_context, 1)?;
					encoder.bit(second_context, 0)?;
					encoder.byte(literal_context, current_byte)?;
				}
			};
		}
	}
}

// =================================================================================================

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 3 {
		println!("Usage: rust-srx <input> <output>");
		exit(0);
	}

	let input: File = File::open(Path::new(&args[1])).unwrap();
	let output: File = File::create(Path::new(&args[2])).unwrap();

	let mut buffered_input = BufReader::with_capacity(1 << 24, input);
	let mut buffered_output = BufWriter::with_capacity(1 << 24, output);

	let mut encoder: StreamEncoder<BufReader<File>, BufWriter<File>> =
		StreamEncoder::new(&mut buffered_input, &mut buffered_output, 24);

	let start: Instant = Instant::now();
	encoder.encode().unwrap();
	let duration: f64 = start.elapsed().as_millis() as f64 / 1000.0;

	let input_size: u64 = buffered_input.stream_position().unwrap();
	let output_size: u64 = buffered_output.stream_position().unwrap();

	println!("{} -> {} ({:.2}%) in {:.2} seconds ({:.2} MB/s)",
		input_size,
		output_size,
		output_size as f64 / input_size as f64 * 100.0,
		duration,
		input_size as f64 / duration / 1024.0 / 1024.0);
}
