#[allow(non_upper_case_globals)]
impl super::Opcode {
    pub const SveCnt: Self = Self(65);
    pub const SveAddvl: Self = Self(66);
    pub const SveAddsvl: Self = Self(67);
    pub const SvePtrue: Self = Self(68);
    pub const SvePtest: Self = Self(69);
    pub const SvePredAnd: Self = Self(70);
    pub const SvePredOrr: Self = Self(71);
    pub const SveMovprfx: Self = Self(72);
    pub const SveDupGpr: Self = Self(73);
    pub const SveAddVec: Self = Self(74);
    pub const SveSubVec: Self = Self(75);
    pub const SveOrrVec: Self = Self(76);
    pub const SveEorVec: Self = Self(77);
    pub const SveSel: Self = Self(78);
    pub const SveLdr: Self = Self(79);
    pub const SveStr: Self = Self(80);
    pub const SveLd1rd: Self = Self(81);
    pub const SveLd1rqd: Self = Self(82);
    pub const SveLd1d: Self = Self(83);
    pub const SveSt1d: Self = Self(84);
    pub const SveSt1b: Self = Self(260);
}
