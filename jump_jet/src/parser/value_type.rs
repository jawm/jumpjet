// TODO: This should probably implement the Parse trait
impl ValueType {
	fn get(key: i64) -> ValueType {
		match key {
			0x01 => ValueType::i_32,
			0x02 => ValueType::i_64,
			0x03 => ValueType::f_32,
			0x04 => ValueType::f_64,
		}
	}
}