use serde::de;
use serde::ser;

use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct CS<T, const N: usize>(pub [T; N]);

impl<T: Default + Copy, const N: usize> Default for CS<T, N> {
    #[inline]
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<T, const N: usize> AsRef<[T]> for CS<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const N: usize> CS<T, N> {
    #[inline]
    pub fn into_inner(self) -> [T; N] {
        self.0
    }

    #[inline]
    pub fn to_inner(&self) -> &[T; N] {
        &self.0
    }
}

impl<T: FromStr + Default + Copy, const N: usize> FromStr for CS<T, N> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut arr = Self::default();
        let it_mut = IntoIterator::into_iter(&mut arr.0);

        let split = s.split(',').filter(|s| !s.is_empty());

        for (entry, s) in it_mut.zip(split) {
            *entry = s.parse()?;
        }
        Ok(arr)
    }
}

impl<T: fmt::Display, const N: usize> fmt::Display for CS<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = IntoIterator::into_iter(&self.0);
        if let Some(v) = it.next() {
            <T as fmt::Display>::fmt(v, f)?;
        }

        for v in it {
            write!(f, ",{}", v)?
        }

        Ok(())
    }
}

impl<T: fmt::Display, const N: usize> ser::Serialize for CS<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T, const N: usize> de::Deserialize<'de> for CS<T, N>
where
    T: FromStr + Default + Copy,
    T::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use std::marker::PhantomData;

        struct CsVisitor<T, const N: usize>(PhantomData<T>);

        impl<'de, T, const N: usize> de::Visitor<'de> for CsVisitor<T, N>
        where
            T: FromStr + Default + Copy,
            T::Err: fmt::Display,
        {
            type Value = CS<T, N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("comma separeted list")
            }

            fn visit_str<E>(self, values: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                values.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(CsVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::CS;
    type CsTest<const N: usize> = CS<u32, N>;

    fn assert_ok_from_str<const N: usize>(s: &str, expected: [u32; N]) {
        let cs: Result<CsTest<N>, _> = s.parse();
        assert!(matches!(cs, Ok(v) if v == CS(expected)))
    }

    fn assert_err_from_str<const N: usize>(s: &str) {
        let cs: Result<CsTest<N>, _> = s.parse();
        assert!(cs.is_err())
    }

    #[test]
    fn from_str() {
        assert_ok_from_str("", []);
        assert_ok_from_str(",,,,", []);

        assert_ok_from_str("1", [1]);
        assert_ok_from_str(",1", [1]);
        assert_ok_from_str("1,", [1]);
        assert_ok_from_str(",,,1,", [1]);

        assert_ok_from_str("1,2", [1, 2]);
        assert_ok_from_str("1,2,3,4,5", [1, 2, 3, 4, 5]);
        assert_ok_from_str("1,,,,,2", [1, 2]);
        assert_ok_from_str(",,,1,,,,,2,,,,,", [1, 2]);

        assert_err_from_str::<1>("-1");
        assert_err_from_str::<2>("1,a,");
    }

    fn assert_to_string<const N: usize>(values: [u32; N], expected: &str) {
        let cs = CS(values).to_string();
        assert_eq!(cs, expected);
    }

    #[test]
    fn to_string() {
        assert_to_string([], "");
        assert_to_string([1], "1");
        assert_to_string([1, 2], "1,2");
        assert_to_string([1, 2, 3, 4, 5], "1,2,3,4,5");
    }

    fn assert_ok_des<const N: usize>(s: &str, expected: [u32; N]) {
        let cs: Result<CsTest<N>, _> = serde_json::from_str(s);
        assert!(matches!(cs, Ok(v) if v == CS(expected)))
    }

    fn assert_err_des<const N: usize>(s: &str) {
        let cs: Result<CsTest<N>, _> = serde_json::from_str(s);
        assert!(cs.is_err())
    }

    #[test]
    fn deserialize() {
        assert_ok_des(r#""""#, []);
        assert_ok_des(r#"",,,,""#, []);

        assert_ok_des(r#""1""#, [1]);
        assert_ok_des(r#"",1""#, [1]);
        assert_ok_des(r#""1,""#, [1]);
        assert_ok_des(r#"",,,1,""#, [1]);

        assert_ok_des(r#""1,2""#, [1, 2]);
        assert_ok_des(r#""1,2,3,4,5""#, [1, 2, 3, 4, 5]);
        assert_ok_des(r#""1,,,,,2""#, [1, 2]);
        assert_ok_des(r#"",,,1,,,,,2,,,,,""#, [1, 2]);

        assert_err_des::<1>(r#""-1""#);
        assert_err_des::<2>(r#""1,a,""#);
    }

    fn assert_ser<const N: usize>(values: [u32; N], expected: &str) {
        let cs = serde_json::to_string(&CS(values));
        assert!(matches!(cs, Ok(v) if v == expected))
    }

    #[test]
    fn serialize() {
        assert_ser([], r#""""#);
        assert_ser([1], r#""1""#);
        assert_ser([1, 2], r#""1,2""#);
        assert_ser([1, 2, 3, 4, 5], r#""1,2,3,4,5""#);
    }
}
