use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

use getrandom::getrandom;

use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

mod common;
use common::from_bytes;
use common::from_vec;

const SMALL_LENGTH: usize = 16;

const LARGE_LENGTH: usize = 1024;

const ITERATIONS: usize = 1024;

fn random_os_string(
    buffer_length: usize,
) -> Result<OsString, getrandom::Error> {
    let mut buffer = vec![0; buffer_length];
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        getrandom(&mut buffer)?;
        Ok(OsStringExt::from_vec(buffer))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        use std::slice;

        getrandom(as_mut_bytes(&mut buffer))?;
        return Ok(OsStringExt::from_wide(&buffer));

        fn as_mut_bytes(buffer: &mut [u16]) -> &mut [u8] {
            // SAFETY: [u16] can always be transmuted to two [u8] bytes.
            unsafe {
                slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    buffer.len() * 2,
                )
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    Err(getrandom::Error::UNSUPPORTED)
}

#[test]
fn test_random_bytes() -> Result<(), getrandom::Error> {
    let os_string = random_os_string(LARGE_LENGTH)?;
    let string = os_string.to_raw_bytes();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(Cow::Borrowed(&*os_string)), from_bytes(&string));
    Ok(())
}

#[test]
fn test_random_vec() -> Result<(), getrandom::Error> {
    let os_string = random_os_string(LARGE_LENGTH)?;
    let string = os_string.clone().into_raw_vec();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(os_string), from_vec(string));
    Ok(())
}

#[test]
fn test_lossless() -> Result<(), getrandom::Error> {
    for _ in 0..ITERATIONS {
        let mut string = vec![0; SMALL_LENGTH];
        getrandom(&mut string)?;
        if let Ok(os_string) = OsStr::from_raw_bytes(&string) {
            let encoded_string = os_string.to_raw_bytes();
            assert_eq!(string, &*encoded_string);
        }
    }
    Ok(())
}

#[cfg(feature = "raw_os_str")]
#[test]
fn test_raw() -> Result<(), getrandom::Error> {
    use os_str_bytes::RawOsStr;
    use os_str_bytes::RawOsString;

    macro_rules! test {
        (
            $result:expr ,
            $method:ident (& $string:ident , & $substring:ident )
        ) => {
            #[allow(clippy::bool_assert_comparison)]
            {
                assert_eq!(
                    $result,
                    $string.$method(&$substring),
                    concat!(stringify!($method), "({:?}, {:?})"),
                    $string,
                    $substring,
                );
            }
        };
    }

    for _ in 0..ITERATIONS {
        let mut string = random_os_string(SMALL_LENGTH)?;
        let prefix = RawOsStr::new(&string).into_owned();
        let suffix = random_os_string(SMALL_LENGTH)?;
        string.push(&suffix);

        let string = RawOsString::new(string);
        let suffix = RawOsString::new(suffix);

        test!(true, ends_with_os(&string, &suffix));
        test!(true, starts_with_os(&string, &prefix));

        if prefix != suffix {
            test!(false, ends_with_os(&string, &prefix));
            test!(false, starts_with_os(&string, &suffix));
        }
    }
    Ok(())
}
