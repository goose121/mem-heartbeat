
#[derive(Copy, Clone, Debug)]
pub enum HeartbeatStage {
	O,
	P,
	// The PR segment
	PR,
	Q,
	R,
	S,
	// The steeply-angled part going to T
	S2,
	// The shallowly-angled part going to T
	ST,
	T,
}

use self::HeartbeatStage::*;

impl HeartbeatStage {
	pub fn duration(&self) -> u64 {
		match *self {
			O => 435,
			P => 80,
			PR => 120,
			Q => 30,
			R => 45,
			S => 10,
			S2 => 20,
			ST => 100,
			T => 160
		}
	}

	pub fn next(self) -> Self {
		match self {
			O => P,
			P => PR,
			PR => Q,
			Q => R,
			R => S,
			S => S2,
			S2 => ST,
			ST => T,
			T => O
		}
	}
}

pub struct Heartbeat {
	stage: HeartbeatStage,
	stage_time: u64
}

const BASE_LEVEL: f64 = 0.2;

fn wave(x: f64) -> f64 {
	x * x - x.powi(3)
}

impl Heartbeat {
	/// Returns a value between 0.0 and 1.0
	/// Amplitude will be 3.4 mV
	pub fn level(&self) -> f64 {
		let adj_time = (self.stage_time as f64) / (self.stage.duration() as f64);
		match self.stage {
			O | PR | ST => BASE_LEVEL,
			// TODO: Adjust amplitude
			P => (wave(adj_time) / 2.0) + BASE_LEVEL,
			Q => adj_time * -(BASE_LEVEL / 2.0) + BASE_LEVEL,
			R => adj_time * (1.0 - BASE_LEVEL / 2.0) + BASE_LEVEL / 2.0,
			S => -adj_time + 1.0,
			S2 => adj_time * BASE_LEVEL,
			//TODO: Adjust amplitude
			T => wave(adj_time) + BASE_LEVEL,
		}
	}

	pub fn inc_time(&mut self, inc: u64) {
		self.stage_time += inc;

		while self.stage_time > self.stage.duration() {
			self.stage_time -= self.stage.duration();
			self.stage = self.stage.next();
		}
	}
}

impl Default for Heartbeat {
	fn default() -> Heartbeat {
		Heartbeat {
			stage: O,
			stage_time: 0
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test::Bencher;

	fn gen_heartbeat(stage: HeartbeatStage) -> Heartbeat {
		Heartbeat {
			stage,
			stage_time: stage.duration() / 2
		}
	}
	
	#[bench]
	fn bench_level_const(b: &mut Bencher) {
		let h = gen_heartbeat(O);
		
		b.iter(|| h.level());
	}

	#[bench]
	fn bench_level_lin(b: &mut Bencher) {
		for lin_stage in &[Q, R, S, S2] {
			let h = gen_heartbeat(*lin_stage);
			
			b.iter(|| h.level());
		}
	}

	#[bench]
	fn bench_level_wave(b: &mut Bencher) {
		for wave_stage in &[P, T] {
			let h = gen_heartbeat(*wave_stage);
			
			b.iter(|| h.level());
		}
	}
}
