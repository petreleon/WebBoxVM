#[allow(non_upper_case_globals)]
impl super::Opcode {
    pub const Mrs: Self = Self(227);
    pub const Msr: Self = Self(228);
    pub const Tlbi: Self = Self(229);
    pub const DcZva: Self = Self(230);
    pub const Svc: Self = Self(231);
    pub const Eret: Self = Self(232);
    pub const Brk: Self = Self(233);
    pub const Rev: Self = Self(234);
    pub const Rev32: Self = Self(235);
    pub const Rev16: Self = Self(236);
    pub const Rbit: Self = Self(237);
    pub const Clz: Self = Self(238);
    pub const Crc32: Self = Self(239);
    pub const Udiv: Self = Self(240);
    pub const Sdiv: Self = Self(241);
    pub const Lslv: Self = Self(242);
    pub const Lsrv: Self = Self(243);
    pub const Asrv: Self = Self(244);
    pub const Rorv: Self = Self(245);
    pub const Extr: Self = Self(246);
    pub const Ldxr: Self = Self(247);
    pub const Ldxp: Self = Self(248);
    pub const Stxr: Self = Self(249);
    pub const Stxp: Self = Self(250);
    pub const Ldar: Self = Self(251);
    pub const Stlr: Self = Self(252);
    pub const Atomic: Self = Self(253);
    pub const AtomicPair: Self = Self(254);
    pub const Cas: Self = Self(255);
    pub const Casp: Self = Self(256);
    pub const Wfi: Self = Self(257);
    pub const Wfe: Self = Self(258);
    pub const Udf: Self = Self(393);
}
