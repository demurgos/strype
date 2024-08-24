#[macro_export]
macro_rules! define_string_type {
  (
    $(#[$struct_meta:meta])*
    $struct_vis:vis struct $struct_name:ident($inner_ty:ty);

    $(macro $macro_name:ident;)?

    $(
      check $err_vis:vis $ck_const:ident $err_name:ident {
        $(
          #[$($ck_meta:tt)*]
          $ck_name:ident,
        )*
      }
    )?
  ) => {
    $(#[$struct_meta])*
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    $struct_vis struct $struct_name<TyInner = $inner_ty>(TyInner);

    $(
      #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
      $err_vis enum $err_name {
        $($ck_name,)*
      }
    )?

    // $(
    //   #[macro_export]
    //   macro_rules! $macro_name {
    //     ($input:expr) => {
    //       {
    //         mod string_check {
    //           #[test]
    //           fn check_macro_value() {
    //             <super::$struct_name as ::core::str::FromStr>::from_str($input).unwrap();
    //           }
    //         }
    //         $struct_name($input)
    //       }
    //     };
    //   }
    // )?

    $crate::define_string_type!(
      @impl_new $struct_name($inner_ty)
      $($err_name($ck_const) {
        $(
          #[$($ck_meta)*]
          $ck_name,
        )*
      })?
    );

    impl<TyInner> $struct_name<TyInner> {
      pub fn as_str(&self) -> &str
        where TyInner: ::core::ops::Deref<Target = str>,
      {
        &*self.0
      }

      pub fn as_view(&self) -> $struct_name<&str>
        where TyInner: ::core::ops::Deref<Target = str>,
      {
        $struct_name(self.as_str())
      }

      pub fn into_inner(self) -> TyInner {
        self.0
      }

      pub const fn as_inner(&self) -> &TyInner {
        &self.0
      }
    }

    impl<'s> $struct_name<&'s str> {
      /// Specialized version of [into_inner] suitable for const context when the inner type is &str.
      ///
      /// Outside of `const` contexts, it is recommended to use `into_inner` or `as_str` directly.
      pub const fn into_inner_str(self) -> &'s str {
        self.0
      }
    }
  };

  // internal rule for method implementation in the case where there are no checks (all strings are valid)
  (@impl_new $struct_name:ident($inner_ty:ty)) => {
    impl<TyInner> $struct_name<TyInner> {
      pub const fn new(inner: TyInner) -> Self
        where TyInner: ::core::ops::Deref<Target = str>,
      {
        Self(inner)
      }
    }

    impl ::core::str::FromStr for $struct_name<$inner_ty> {
      type Err = ::core::convert::Infallible;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(<$inner_ty>::from(s)))
      }
    }
  };

  // internal rule for method implementation in the case where there are const checks (all strings are not valid)
  (
    @impl_new $struct_name:ident($inner_ty:ty)
    $err_name:ident(const) {
      $(
        #[$($ck_meta:tt)*]
        $ck_name:ident,
      )*
    }
  ) => {
    impl<TyInner> $struct_name<TyInner> {
      pub fn new(input: TyInner) -> Result<Self, $err_name>
        where TyInner: ::core::ops::Deref<Target = str>,
      {
        match $struct_name::check(&*input) {
          Ok(_) => Ok(Self(input)),
          Err(e) => Err(e),
        }
      }
    }

    impl<'s> $struct_name<&'s str>
    {
      /// Specialized new for `&'s str` intended for const contexts. Prefer `new` when outside a
      /// const context.
      pub const fn check(input: &'s str) -> Result<Self, $err_name> {
        $(
          $crate::define_string_type!(@check $err_name::$ck_name($($ck_meta)*)(input));
        )*
        Ok(Self(input))
      }
    }

    impl ::core::str::FromStr for $struct_name<$inner_ty> {
      type Err = $err_name;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(<$inner_ty>::from(s))
      }
    }
  };

  // internal rule for method implementation in the case where there are dyn (non-const) checks (all strings are not valid)
  (
    @impl_new $struct_name:ident($inner_ty:ty)
    $err_name:ident(dyn) {
      $(
        #[$($ck_meta:tt)*]
        $ck_name:ident,
      )*
    }
  ) => {
    impl<TyInner> $struct_name<TyInner> {
      pub fn new(input: TyInner) -> Result<Self, $err_name>
        where TyInner: ::core::ops::Deref<Target = str>,
      {
        match $struct_name::check(&*input) {
          Ok(_) => Ok(Self(input)),
          Err(e) => Err(e),
        }
      }
    }

    impl<'s> $struct_name<&'s str>
    {
      /// Specialized new for `&'s str`.
      pub fn check(input: &'s str) -> Result<Self, $err_name> {
        $(
          $crate::define_string_type!(@check $err_name::$ck_name($($ck_meta)*)(input));
        )*
        Ok(Self(input))
      }
    }

    impl ::core::str::FromStr for $struct_name<$inner_ty> {
      type Err = $err_name;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(<$inner_ty>::from(s))
      }
    }
  };

  (@check $err_name:ident::$ck_name:ident(non_empty)($input:expr)) => {
    if $input.is_empty() {
      return Err($err_name::$ck_name);
    }
  };

  (@check $err_name:ident::$ck_name:ident(ascii_trimmed)($input:expr)) => {
    if $input.trim_ascii().len() != $input.len() {
      return Err($err_name::$ck_name);
    }
  };

  (@check $err_name:ident::$ck_name:ident(min_len($l:expr))($input:expr)) => {
    if $input.len() < $l {
      return Err($err_name::$ck_name);
    }
  };

  (@check $err_name:ident::$ck_name:ident(max_len($l:expr))($input:expr)) => {
    if $input.len() > $l {
      return Err($err_name::$ck_name);
    }
  };

  (@check $err_name:ident::$ck_name:ident(len($l:expr))($input:expr)) => {
    if $input.len() != $l {
      return Err($err_name::$ck_name);
    }
  };

  (@check $err_name:ident::$ck_name:ident(regex($pattern:expr))($input:expr)) => {
    #[allow(clippy::trivial_regex)]
    static PATTERN: ::std::sync::LazyLock<::regex::Regex> = ::std::sync::LazyLock::new(|| {
      let pat: &str = $pattern;
      match ::regex::Regex::new(pat) {
        Ok(pat) => pat,
        Err(e) => panic!("regex check {}::{} pattern {pat:?} should be valid: {e}", stringify!($err_name), stringify!($ck_name)),
      }
    });
    if !PATTERN.is_match($input) {
      return Err($err_name::$ck_name);
    }
  };
}

#[macro_export]
macro_rules! declare_new_string {
  (
    $(#[$struct_meta:meta])*
    $struct_vis:vis struct $struct_name:ident(String);
    $(#[$err_meta:meta])*
    $err_vis:vis type ParseError = $err_name:ident;
    const PATTERN = $pattern:expr;
    $(const SQL_NAME = $sql_name:literal;)?
  ) => {
    $(#[$err_meta:meta])*
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $err_name(());

    impl ::std::fmt::Display for $err_name {
      fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(concat!("Invalid ", stringify!($struct_name)), fmt)
      }
    }

    impl ::std::error::Error for $err_name {}

    $(#[$struct_meta])*
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    $struct_vis struct $struct_name(String);

    impl $struct_name {
      $struct_vis fn pattern() -> &'static ::regex::Regex {
        #[allow(clippy::trivial_regex)]
        static PATTERN: ::once_cell::sync::Lazy<::regex::Regex> = ::once_cell::sync::Lazy::new(||
          ::regex::Regex::new($pattern).unwrap()
        );
        &*PATTERN
      }

      #[inline]
      $struct_vis fn as_str(&self) -> &str {
        &self.0
      }
    }

    impl ::std::fmt::Display for $struct_name {
      fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.0, fmt)
      }
    }

    impl ::std::str::FromStr for $struct_name {
      type Err = $err_name;

      fn from_str(s: &str) ->  ::std::result::Result<Self, Self::Err> {
        if Self::pattern().is_match(&s) {
          Ok(Self(s.to_string()))
        } else {
          Err($err_name(()))
        }
      }
    }

    impl ::std::convert::TryFrom<&str> for $struct_name {
      type Error = $err_name;

      fn try_from(s: &str) ->  ::std::result::Result<Self, Self::Error> {
        s.parse()
      }
    }

    #[cfg(feature="serde")]
    impl ::serde::Serialize for $struct_name {
      fn serialize<S: ::serde::Serializer>(&self, serializer: S) ->  ::std::result::Result<S::Ok, S::Error> {
        ::serde::Serialize::serialize(self.as_str(), serializer)
      }
    }

    #[cfg(feature="serde")]
    impl<'de> ::serde::Deserialize<'de> for $struct_name {
      fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) ->  ::std::result::Result<Self, D::Error> {
        struct SerdeVisitor;
        impl<'de> ::serde::de::Visitor<'de> for SerdeVisitor {
          type Value = $struct_name;

          fn expecting(&self, fmt: &mut ::std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str(concat!("a string for a valid ", stringify!($struct_name)))
          }

          fn visit_str<E: ::serde::de::Error>(self, value: &str) ->  ::std::result::Result<Self::Value, E> {
            value.parse().map_err(E::custom)
          }
        }

        deserializer.deserialize_str(SerdeVisitor)
      }
    }

    $($crate::declare_new_string! {
      @impl_sqlx $struct_name $sql_name
    })?
  };

  (@impl_sqlx $struct_name:ident $sql_name:literal) => {
    #[cfg(feature = "sqlx-postgres")]
    impl ::sqlx::Type<sqlx::Postgres> for $struct_name {
      fn type_info() -> ::sqlx::postgres::PgTypeInfo {
        ::sqlx::postgres::PgTypeInfo::with_name($sql_name)
      }

      fn compatible(ty: &::sqlx::postgres::PgTypeInfo) -> bool {
        *ty == Self::type_info() || <&str as ::sqlx::Type<::sqlx::Postgres>>::compatible(ty)
      }
    }

    #[cfg(feature = "sqlx")]
    impl<'r, Db: ::sqlx::Database> ::sqlx::Decode<'r, Db> for $struct_name
    where
      &'r str: ::sqlx::Decode<'r, Db>,
    {
      fn decode(
        value: <Db as ::sqlx::database::HasValueRef<'r>>::ValueRef,
      ) ->  ::std::result::Result<Self, Box<dyn ::std::error::Error + 'static + Send + Sync>> {
        let value: &str = <&str as ::sqlx::Decode<Db>>::decode(value)?;
        Ok(value.parse()?)
      }
    }

    // Can't implement generically over `sqlx::Database` because of lifetime issues.
    #[cfg(feature = "sqlx-postgres")]
    impl ::sqlx::Encode<'_, ::sqlx::Postgres> for $struct_name {
      fn encode_by_ref(&self, buf: &mut ::sqlx::postgres::PgArgumentBuffer) -> ::sqlx::encode::IsNull {
        <&str as sqlx::Encode<'_, ::sqlx::Postgres>>::encode(self.as_str(), buf)
      }
    }
  };
}
//
//
// declare_new_string! {
//   pub struct UserDisplayName(String);
//   pub type ParseError = UserDisplayNameParseError;
//   const PATTERN = r"^[\p{Letter}_][\p{Letter}_ ()0-9]{0,62}[\p{Letter}_()0-9]$";
//   const SQL_NAME = "user_display_name";
// }