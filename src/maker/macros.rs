macro_rules! create_rule {
    // entries
    [tiff {$($body:tt)*}] => {
        $crate::tiff::ExifTask::Tiff(create_rule![@acc() $($body)*])
    };
    [template {$($body:tt)*}] => {
        $crate::tiff::ExifTask::Offset($crate::tiff::OffsetType::Bytes(0), create_rule![@acc() $($body)*])
    };

    // blocks
    [@acc($($x:tt)*) tiff {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Tiff(create_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) next {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::JumpNext(create_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) load($name:ident) $($tails:tt)*] => {
        create_rule![@acc($($x)* $name.clone(),) $($tails)*]
    };

    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Condition {
            cond: ($crate::tiff::CondType::EXIST, stringify!($a), 0),
            left: create_rule![@acc() $($body1)*],
            right: create_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)*) if $a ? {$($body1)*} else {} $($tails)*]
    };

    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Condition {
            cond: ($crate::tiff::CondType::EQ, stringify!($a), $b),
            left: create_rule![@acc() $($body1)*],
            right: create_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)*) if $a == $b {$($body1)*} else {} $($tails)*]
    };

    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Condition {
            cond: ($crate::tiff::CondType::LT, stringify!($a), $b),
            left: create_rule![@acc() $($body1)*],
            right: create_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)*) if $a < $b {$($body1)*} else {} $($tails)*]
    };

    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Condition {
            cond: ($crate::tiff::CondType::GT, stringify!($a), $b),
            left: create_rule![@acc() $($body1)*],
            right: create_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)*) if $a > $b {$($body1)*} else {} $($tails)*]
    };

    [@acc($($x:tt)*) scan [$($marker:tt)*] / $name:ident {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Scan {
            marker: &[$($marker)*],
            name: Some(stringify!($name)),
            tasks: create_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) scan [$($marker:tt)*] {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Scan {
            target: &[$($marker)*],
            name: None,
            tasks: create_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    [@acc($($x:tt)*) offset address {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Offset($crate::tiff::OffsetType::Address, create_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) offset + $offset:ident {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Offset($crate::tiff::OffsetType::PrevField(stringify!($offset)), create_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) offset + $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Offset($crate::tiff::OffsetType::Bytes($offset), create_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) offset - $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Offset($crate::tiff::OffsetType::Bytes(-$offset), create_rule![@acc() $($body)*]),) $($tails)*]
    };

    [@acc($($x:tt)*) sony_decrypt / $offset_tag:tt / $len_tag:tt / $key_tag:tt {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::SonyDecrypt {
            offset_tag: $offset_tag,
            len_tag: $len_tag,
            key_tag: $key_tag,
            tasks: create_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // these rules must be the lastest of blocks
    [@acc($($x:tt)*) $tag:tt / $name:tt {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)*) $tag / $name $tag {$($body)*} $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt ? {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Jump {
            tag: $tag,
            is_optional: true,
            tasks: create_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt {$($body:tt)*} $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::Jump {
            tag: $tag,
            is_optional: false,
            tasks: create_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // data types
    [@acc($($x:tt)*) $tag:tt / $name:tt($len:tt) $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::TagItem {
            tag: $tag,
            name: create_rule!(@name $name),
            len: Some(create_rule!(@name $len)),
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt ? / $name:tt($len:tt) $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::TagItem {
            tag: $tag,
            name: create_rule!(@name $name),
            len: Some(create_rule!(@name $len)),
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt ? / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::TagItem {
            tag: $tag,
            name: create_rule!(@name $name),
            len: None,
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::TagItem {
            tag: $tag,
            name: create_rule!(@name $name),
            len: None,
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    [@acc($($x:tt)*) $tag:tt : u16 / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::TagItem {
            tag: $tag,
            name: create_rule!(@name $name),
            len: None,
            is_optional: false,
            is_value_u16: true
        },) $($tails)*]
    };

    // with offset
    [@acc($($x:tt)*) u16 + $offset:tt / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::OffsetItem {
            offset: $offset,
            name: create_rule!(@name $name),
            t: $crate::tiff::Value::U16(0)
        },) $($tails)*]
    };
    [@acc($($x:tt)*) u32 + $offset:tt / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::OffsetItem {
            offset: $offset,
            name: create_rule!(@name $name),
            t: $crate::tiff::Value::U32(0)
        },) $($tails)*]
    };
    [@acc($($x:tt)*) r64 + $offset:tt / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::OffsetItem {
            offset: $offset,
            name: create_rule!(@name $name),
            t: $crate::tiff::Value::R64(0.)
        },) $($tails)*]
    };
    [@acc($($x:tt)*) str + $offset:tt / $name:tt $($tails:tt)*] => {
        create_rule![@acc($($x)* $crate::tiff::ExifTask::OffsetItem {
            offset: $offset,
            name: create_rule!(@name $name),
            t: $crate::tiff::Value::Str("".to_owned())
        },) $($tails)*]
    };

    // reduce
    [@acc($($x:tt)*)] => {
        vec![$($x)*]
    };

    // possible fields
    [@name $name:tt] => { stringify!($name) };
}
