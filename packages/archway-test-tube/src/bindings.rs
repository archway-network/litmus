use test_tube::bindings::GoUint64;

extern "C" {
    pub fn SkipBlock(envId: GoUint64);
}
