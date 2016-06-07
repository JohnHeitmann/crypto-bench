#![feature(test)]

extern crate test;

#[macro_use]
extern crate crypto_bench;

extern crate sodiumoxide;

mod aead;

mod digest {
    macro_rules! sodiumoxide_digest_benches {
        ( $name:ident, $block_len:expr, $output_len:expr, $digester:path) => {
            mod $name {
                use crypto_bench;
                use $digester;

                digest_benches!($block_len, input, {
                    hash(&input)
                });
            }
        }
    }

    sodiumoxide_digest_benches!(
        sha256, crypto_bench::SHA256_BLOCK_LEN, crypto_bench::SHA256_OUTPUT_LEN,
        sodiumoxide::crypto::hash::sha256::hash);
    sodiumoxide_digest_benches!(
        sha512, crypto_bench::SHA512_BLOCK_LEN, crypto_bench::SHA512_OUTPUT_LEN,
        sodiumoxide::crypto::hash::sha512::hash);
}