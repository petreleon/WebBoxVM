#[allow(non_upper_case_globals)]
impl super::Opcode {
    pub const SimdAese: Self = Self(101);
    pub const SimdAesd: Self = Self(102);
    pub const SimdAesmc: Self = Self(103);
    pub const SimdAesimc: Self = Self(104);
    pub const SimdPmull: Self = Self(105);
    pub const SimdSha1h: Self = Self(106);
    pub const SimdSha256Su0: Self = Self(107);
    pub const SimdSha512Su0: Self = Self(108);
    pub const SimdSha512H: Self = Self(429);
    pub const SimdSha512H2: Self = Self(430);
    pub const SimdSha512Su1: Self = Self(431);
    pub const SimdSha1C: Self = Self(435);
    pub const SimdSha1M: Self = Self(436);
    pub const SimdSha1P: Self = Self(437);
    pub const SimdSha1Su0: Self = Self(438);
    pub const SimdSha1Su1: Self = Self(439);
    pub const SimdSha256H: Self = Self(440);
    pub const SimdSha256H2: Self = Self(441);
    pub const SimdSha256Su1: Self = Self(442);
    pub const SimdSm4e: Self = Self(109);
    pub const SimdSm4EKey: Self = Self(455);
    pub const SimdSm3Partw1: Self = Self(110);
    pub const SimdSm3Partw2: Self = Self(445);
    pub const SimdSm3Ss1: Self = Self(446);
    pub const SimdSm3Tt1A: Self = Self(447);
    pub const SimdSm3Tt1B: Self = Self(448);
    pub const SimdSm3Tt2A: Self = Self(449);
    pub const SimdSm3Tt2B: Self = Self(450);
    pub const SimdEor3: Self = Self(111);
    pub const SimdBcax: Self = Self(112);
    pub const SimdRax1: Self = Self(113);
    pub const SimdXar: Self = Self(114);
}
