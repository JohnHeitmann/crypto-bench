macro_rules! sodiumoxide_seal_in_place_bench {
    ( $benchmark_name:ident, $sealer:path,
      $guard:expr, $input_len:expr, $ad:expr ) => {

        #[bench]
        fn $benchmark_name(b: &mut test::Bencher) {
            use $sealer as _sealer;
            use sodiumoxide;

            sodiumoxide::init();

            if !$guard {
                return;
            }

            b.bytes = $input_len as u64;
            let key = _sealer::gen_key();
            let nonce = _sealer::gen_nonce();
            let mut in_out = vec![0u8; $input_len + _sealer::MACBYTES];
            b.iter(|| {
                _sealer::encrypt_in_place(&mut in_out, $input_len,
                                          $ad, &nonce, &key).unwrap()
            });
        }
    }
}

macro_rules! sodiumoxide_aead_benchs {
    ( $name:ident, $sealer:path, $guard:expr ) => {
        mod $name {
            use crypto_bench;
            use test;

            // A TLS 1.2 finished message.
            sodiumoxide_seal_in_place_bench!(
                tls12_finished, $sealer, $guard,
                crypto_bench::aead::TLS12_FINISHED_LEN,
                &crypto_bench::aead::TLS12_AD);
            sodiumoxide_seal_in_place_bench!(
                tls13_finished, $sealer, $guard,
                crypto_bench::aead::TLS13_FINISHED_LEN,
                &crypto_bench::aead::TLS13_AD);

            // For comparison with BoringSSL.
            sodiumoxide_seal_in_place_bench!(
                tls12_16, $sealer, $guard, 16,
                &crypto_bench::aead::TLS12_AD);

            // ~1 packet of data in TLS.
            sodiumoxide_seal_in_place_bench!(
                tls12_1350, $sealer, $guard, 1350,
                &crypto_bench::aead::TLS12_AD);
            sodiumoxide_seal_in_place_bench!(
                tls13_1350, $sealer, $guard, 1350,
                &crypto_bench::aead::TLS13_AD);

            // For comparison with BoringSSL.
            sodiumoxide_seal_in_place_bench!(
                tls12_8192, $sealer, $guard, 8192,
                &crypto_bench::aead::TLS12_AD);
            sodiumoxide_seal_in_place_bench!(
                tls13_8192, $sealer, $guard, 8192,
                &crypto_bench::aead::TLS13_AD);
        }
    }
}

sodiumoxide_aead_benchs!(chacha20_poly1305,
                         sodiumoxide::crypto::aead::chacha20poly1305,
                         true);
sodiumoxide_aead_benchs!(aes_256_gcm,
                         sodiumoxide::crypto::aead::aes256gcm,
                         sodiumoxide::crypto::aead::aes256gcm::is_available());
