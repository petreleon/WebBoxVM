pub(super) fn single_atomic(m: disarm64::decoder::Mnemonic) -> bool {
    swp(m) || ldadd(m) || ldclr(m) || ldeor(m) || ldset(m) || signed_minmax(m) || unsigned_minmax(m)
}

fn swp(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#swp
            | M::r#swpa
            | M::r#swpal
            | M::r#swpl
            | M::r#swpab
            | M::r#swpah
            | M::r#swpalb
            | M::r#swpalh
            | M::r#swpb
            | M::r#swph
            | M::r#swplb
            | M::r#swplh
    )
}

fn ldadd(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldadd
            | M::r#ldadda
            | M::r#ldaddal
            | M::r#ldaddl
            | M::r#ldaddab
            | M::r#ldaddah
            | M::r#ldaddalb
            | M::r#ldaddalh
            | M::r#ldaddb
            | M::r#ldaddh
            | M::r#ldaddlb
            | M::r#ldaddlh
    )
}

fn ldclr(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldclr
            | M::r#ldclra
            | M::r#ldclral
            | M::r#ldclrl
            | M::r#ldclrab
            | M::r#ldclrah
            | M::r#ldclralb
            | M::r#ldclralh
            | M::r#ldclrb
            | M::r#ldclrh
            | M::r#ldclrlb
            | M::r#ldclrlh
    )
}

fn ldeor(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldeor
            | M::r#ldeora
            | M::r#ldeoral
            | M::r#ldeorl
            | M::r#ldeorab
            | M::r#ldeorah
            | M::r#ldeoralb
            | M::r#ldeoralh
            | M::r#ldeorb
            | M::r#ldeorh
            | M::r#ldeorlb
            | M::r#ldeorlh
    )
}

fn ldset(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldset
            | M::r#ldseta
            | M::r#ldsetal
            | M::r#ldsetl
            | M::r#ldsetab
            | M::r#ldsetah
            | M::r#ldsetalb
            | M::r#ldsetalh
            | M::r#ldsetb
            | M::r#ldseth
            | M::r#ldsetlb
            | M::r#ldsetlh
    )
}

fn signed_minmax(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldsmax
            | M::r#ldsmaxa
            | M::r#ldsmaxal
            | M::r#ldsmaxl
            | M::r#ldsmin
            | M::r#ldsmina
            | M::r#ldsminal
            | M::r#ldsminl
    )
}

fn unsigned_minmax(m: disarm64::decoder::Mnemonic) -> bool {
    use disarm64::decoder::Mnemonic as M;
    matches!(
        m,
        M::r#ldumax
            | M::r#ldumaxa
            | M::r#ldumaxal
            | M::r#ldumaxl
            | M::r#ldumin
            | M::r#ldumina
            | M::r#lduminal
            | M::r#lduminl
    )
}
