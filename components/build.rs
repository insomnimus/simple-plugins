// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::{
	env,
	fmt::Write,
	fs,
	io,
	path::PathBuf,
};

fn push_names(buf: &mut String, count: usize, prefix: &str, suffix: &str) {
	*buf += prefix;

	for i in 0..count {
		if i > 0 {
			*buf += ", ";
		}
		write!(buf, "C{}", i + 1).unwrap();
	}

	*buf += suffix;
}

fn push_bounds(buf: &mut String, bound: &str, count: usize, prefix: &str, suffix: &str) {
	*buf += prefix;

	for i in 0..count {
		if i > 0 {
			*buf += ",\n";
		}
		write!(buf, "C{}: {}", i + 1, bound).unwrap();
	}

	*buf += suffix;
}

fn push_impl_header(buf: &mut String, tuple_len: usize, trait_name: &str) {
	push_names(buf, tuple_len, "impl<", ">");
	write!(buf, "{trait_name} for ").unwrap();
	push_names(buf, tuple_len, "(", ")\n");
	push_bounds(buf, trait_name, tuple_len, "where\n", "");
	*buf += "\n{\n";
}

fn main() -> io::Result<()> {
	let mut out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
	out.push("component_impls.rs");

	let mut buf = String::with_capacity(8 << 10);
	for n in 2..=8 {
		// ComponentMeta
		push_impl_header(&mut buf, n, "ComponentMeta");
		buf += "fn reset(&mut self) {\n";
		for i in 0..n {
			writeln!(buf, "\tself.{i}.reset();").unwrap();
		}

		buf += "}\nfn latency(&self) -> usize {\n";
		for i in 0..n {
			if i > 0 {
				buf += " + ";
			}
			write!(buf, "self.{i}.latency()").unwrap();
		}
		buf += "}\n}\n\n";

		// Component
		push_impl_header(&mut buf, n, "Component");
		buf += "fn process(&mut self, mut sample: f64) -> f64 {\n";
		for i in 0..n {
			writeln!(buf, "sample = self.{i}.process(sample);").unwrap();
		}
		buf += "\nsample\n}\n}";

		// ComponentBlock
		push_impl_header(&mut buf, n, "ComponentBlock");
		buf += "fn process_block(&mut self, samples: &mut [f32]) {\n";
		for i in 0..n {
			writeln!(buf, "self.{i}.process_block(samples);").unwrap();
		}
		buf += "}\n}\n";
	}

	fs::write(out, buf.as_bytes())?;
	Ok(())
}
