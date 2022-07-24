use serde::de;
use serde::ser;

use std::str::FromStr;
use std::{fmt, vec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CS<T>(pub Vec<T>);

impl<T> Default for CS<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> AsRef<[T]> for CS<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> From<Vec<T>> for CS<T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> CS<T> {
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    #[inline]
    pub fn to_inner(&self) -> &Vec<T> {
        &self.0
    }

    #[inline]
    pub fn to_inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T: FromStr> FromStr for CS<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .filter(|s| !s.is_empty())
            .map(T::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

impl<T> IntoIterator for CS<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: fmt::Display> fmt::Display for CS<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.iter();
        if let Some(v) = it.next() {
            <T as fmt::Display>::fmt(v, f)?;
        }

        for v in it {
            write!(f, ",{}", v)?
        }

        Ok(())
    }
}

impl<T: fmt::Display> ser::Serialize for CS<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> de::Deserialize<'de> for CS<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use std::marker::PhantomData;

        struct CsVisitor<T>(PhantomData<T>);

        impl<'de, T> de::Visitor<'de> for CsVisitor<T>
        where
            T: FromStr,
            T::Err: fmt::Display,
        {
            type Value = CS<T>;

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
    type CsTest = CS<u32>;

    fn assert_ok_from_str(s: &str, expected: Vec<u32>) {
        let cs: Result<CsTest, _> = s.parse();
        assert!(matches!(cs, Ok(v) if v == CS(expected)))
    }

    fn assert_err_from_str(s: &str) {
        let cs: Result<CsTest, _> = s.parse();
        assert!(cs.is_err())
    }

    #[test]
    fn from_str() {
        assert_ok_from_str("", vec![]);
        assert_ok_from_str(",,,,", vec![]);

        assert_ok_from_str("1", vec![1]);
        assert_ok_from_str(",1", vec![1]);
        assert_ok_from_str("1,", vec![1]);
        assert_ok_from_str(",,,1,", vec![1]);

        assert_ok_from_str("1,2", vec![1, 2]);
        assert_ok_from_str("1,2,3,4,5", vec![1, 2, 3, 4, 5]);
        assert_ok_from_str("1,,,,,2", vec![1, 2]);
        assert_ok_from_str(",,,1,,,,,2,,,,,", vec![1, 2]);

        assert_err_from_str("-1");
        assert_err_from_str("1,a,");
    }

    fn assert_to_string(values: Vec<u32>, expected: &str) {
        let cs = CS(values).to_string();
        assert_eq!(cs, expected);
    }

    #[test]
    fn to_string() {
        assert_to_string(vec![], "");
        assert_to_string(vec![1], "1");
        assert_to_string(vec![1, 2], "1,2");
        assert_to_string(vec![1, 2, 3, 4, 5], "1,2,3,4,5");
    }

    fn assert_ok_des(s: &str, expected: Vec<u32>) {
        let cs: Result<CsTest, _> = serde_json::from_str(s);
        assert!(matches!(cs, Ok(v) if v == CS(expected)))
    }

    fn assert_err_des(s: &str) {
        let cs: Result<CsTest, _> = serde_json::from_str(s);
        assert!(cs.is_err())
    }

    #[test]
    fn deserialize() {
        assert_ok_des(r#""""#, vec![]);
        assert_ok_des(r#"",,,,""#, vec![]);

        assert_ok_des(r#""1""#, vec![1]);
        assert_ok_des(r#"",1""#, vec![1]);
        assert_ok_des(r#""1,""#, vec![1]);
        assert_ok_des(r#"",,,1,""#, vec![1]);

        assert_ok_des(r#""1,2""#, vec![1, 2]);
        assert_ok_des(r#""1,2,3,4,5""#, vec![1, 2, 3, 4, 5]);
        assert_ok_des(r#""1,,,,,2""#, vec![1, 2]);
        assert_ok_des(r#"",,,1,,,,,2,,,,,""#, vec![1, 2]);

        assert_err_des(r#""-1""#);
        assert_err_des(r#""1,a,""#);
    }

    fn assert_ser(values: Vec<u32>, expected: &str) {
        let cs = serde_json::to_string(&CS(values));
        assert!(matches!(cs, Ok(v) if v == expected))
    }

    #[test]
    fn serialize() {
        assert_ser(vec![], r#""""#);
        assert_ser(vec![1], r#""1""#);
        assert_ser(vec![1, 2], r#""1,2""#);
        assert_ser(vec![1, 2, 3, 4, 5], r#""1,2,3,4,5""#);
    }
}
