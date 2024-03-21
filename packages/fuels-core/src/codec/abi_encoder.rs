mod bounded_encoder;
#[cfg(not(experimental))]
mod experimental_bounded_encoder;

use std::default::Default;

#[cfg(not(experimental))]
use crate::codec::abi_encoder::experimental_bounded_encoder::ExperimentalBoundedEncoder;
use crate::{
    codec::abi_encoder::bounded_encoder::BoundedEncoder,
    types::{errors::Result, unresolved_bytes::UnresolvedBytes, Token},
};

#[derive(Debug, Clone, Copy)]
pub struct EncoderConfig {
    /// Entering a struct, array, tuple, enum or vector increases the depth. Encoding will fail if
    /// the current depth becomes greater than `max_depth` configured here.
    pub max_depth: usize,
    /// Every encoded argument will increase the token count. Encoding will fail if the current
    /// token count becomes greater than `max_tokens` configured here.
    pub max_tokens: usize,
    /// The total memory size of the top-level token must fit in the available memory of the
    /// system.
    pub max_total_enum_width: usize,
}

// ANCHOR: default_encoder_config
impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            max_depth: 45,
            max_tokens: 10_000,
            max_total_enum_width: 10_000,
        }
    }
}
// ANCHOR_END: default_encoder_config

#[derive(Default, Clone, Debug)]
pub struct ABIEncoder {
    pub config: EncoderConfig,
}

impl ABIEncoder {
    pub fn new(config: EncoderConfig) -> Self {
        Self { config }
    }

    /// Encodes `Token`s in `args` following the ABI specs defined
    /// [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md)
    pub fn encode(&self, args: &[Token]) -> Result<UnresolvedBytes> {
        #[cfg(experimental)]
        let res = BoundedEncoder::new(self.config, false).encode(args);
        #[cfg(not(experimental))]
        let res = ExperimentalBoundedEncoder::new(self.config, false).encode(args);

        res
    }
}

#[derive(Default, Clone, Debug)]
pub struct ConfigurablesEncoder {
    pub config: EncoderConfig,
}

impl ConfigurablesEncoder {
    pub fn new(config: EncoderConfig) -> Self {
        Self { config }
    }

    /// Encodes `Token`s in `args` following the ABI specs defined
    /// [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md)
    pub fn encode(&self, args: &[Token]) -> Result<UnresolvedBytes> {
        BoundedEncoder::new(self.config, true).encode(args)
    }
}

#[cfg(test)]
mod tests {
    use std::slice;

    #[cfg(experimental)]
    use itertools::chain;
    #[cfg(experimental)]
    use sha2::{Digest, Sha256};

    use super::*;
    #[cfg(experimental)]
    use crate::codec::first_four_bytes_of_sha256_hash;
    #[cfg(experimental)]
    use crate::constants::WORD_SIZE;
    use crate::{
        to_named,
        types::{
            errors::Error,
            param_types::{EnumVariants, ParamType},
            StaticStringToken, U256,
        },
    };

    #[cfg(experimental)]
    const VEC_METADATA_SIZE: usize = 3 * WORD_SIZE;
    #[cfg(experimental)]
    const DISCRIMINANT_SIZE: usize = WORD_SIZE;

    #[test]
    #[cfg(experimental)]
    fn encode_function_signature() {
        let fn_signature = "entry_one(u64)";

        let result = first_four_bytes_of_sha256_hash(fn_signature);

        println!("Encoded function selector for ({fn_signature}): {result:#0x?}");

        assert_eq!(result, [0x0, 0x0, 0x0, 0x0, 0x0c, 0x36, 0xcb, 0x9c]);
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_u32_type() -> Result<()> {
        // @todo eventually we must update the json abi examples in here.
        // They're in the old format.
        //
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u32"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "entry_one(u32)";
        let arg = Token::U32(u32::MAX);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xb7, 0x9e, 0xf7, 0x43];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_u32_type_multiple_args() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"u32"}],
        //         "name":"takes_two",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_two(u32,u32)";
        let first = Token::U32(u32::MAX);
        let second = Token::U32(u32::MAX);

        let args: Vec<Token> = vec![first, second];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff,
        ];

        let expected_fn_selector = [0x0, 0x0, 0x0, 0x0, 0xa7, 0x07, 0xb0, 0x8e];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);
        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_fn_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_u64_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u64"}],
        //         "name":"entry_one",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "entry_one(u64)";
        let arg = Token::U64(u64::MAX);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x0c, 0x36, 0xcb, 0x9c];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_bool_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"bool"}],
        //         "name":"bool_check",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "bool_check(bool)";
        let arg = Token::Bool(true);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x66, 0x8f, 0xff, 0x58];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_two_different_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"first","type":"u32"},{"name":"second","type":"bool"}],
        //         "name":"takes_two_types",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_two_types(u32,bool)";
        let first = Token::U32(u32::MAX);
        let second = Token::Bool(true);

        let args: Vec<Token> = vec![first, second];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, // u32::MAX
            0x1,  // true
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xf5, 0x40, 0x73, 0x2b];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_bits256_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"b256"}],
        //         "name":"takes_bits256",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_bits256(b256)";

        let mut hasher = Sha256::new();
        hasher.update("test string".as_bytes());

        let arg = hasher.finalize();

        let arg = Token::B256(arg.into());

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f, 0x18, 0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44,
            0xe4, 0xcb, 0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, 0xa8, 0xf8, 0x27, 0x43,
            0xf3, 0x1e, 0x93, 0xb,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x01, 0x49, 0x42, 0x96];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_array_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"u8[3]"}],
        //         "name":"takes_integer_array",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_integer_array(u8[3])";

        // Keeping the construction of the arguments array separate for better readability.
        let first = Token::U8(1);
        let second = Token::U8(2);
        let third = Token::U8(3);

        let arg = vec![first, second, third];
        let arg_array = Token::Array(arg);

        let args: Vec<Token> = vec![arg_array];

        let expected_encoded_abi = [0x1, 0x2, 0x3, 0x0, 0x0, 0x0, 0x0, 0x0];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x2c, 0x5a, 0x10, 0x2e];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_string_array_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"str[23]"}],
        //         "name":"takes_string",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_string(str[23])";

        let args: Vec<Token> = vec![Token::StringArray(StaticStringToken::new(
            "This is a full sentence".into(),
            Some(23),
        ))];

        let expected_encoded_abi = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c,
            0x20, 0x73, 0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, 0x0,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xd5, 0x6e, 0x76, 0x51];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_string_slice_type() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"str"}],
        //         "name":"takes_string",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_string(str)";

        let args: Vec<Token> = vec![Token::StringSlice(StaticStringToken::new(
            "This is a full sentence".into(),
            None,
        ))];

        let expected_encoded_abi = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, // str at data index 16
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x17, // str of lenght 23
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, //
            0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c, 0x20, 0x73, //
            0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, //
        ];

        let expected_function_selector = [0, 0, 0, 0, 239, 77, 222, 230];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_struct() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"MyStruct"}],
        //         "name":"takes_my_struct",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_my_struct(MyStruct)";

        // struct MyStruct {
        //     foo: u8,
        //     bar: bool,
        // }

        let foo = Token::U8(1);
        let bar = Token::Bool(true);

        // Create the custom struct token using the array of tuples above
        let arg = Token::Struct(vec![foo, bar]);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0x1, // 1u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // padding
            0x1, // true
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // padding
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xa8, 0x1e, 0x8d, 0xd7];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_enum() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"MyEnum"}],
        //         "name":"takes_my_enum",
        //         "outputs": []
        //     }
        // ]
        // "#;

        let fn_signature = "takes_my_enum(MyEnum)";

        // enum MyEnum {
        //     x: u32,
        //     y: bool,
        // }
        let types = to_named(&[ParamType::U32, ParamType::Bool]);
        let params = EnumVariants::new(types)?;

        // An `EnumSelector` indicating that we've chosen the first Enum variant,
        // whose value is 42 of the type ParamType::U32 and that the Enum could
        // have held any of the other types present in `params`.

        let enum_selector = Box::new((0, Token::U32(42), params));

        let arg = Token::Enum(enum_selector);

        let args: Vec<Token> = vec![arg];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x35, 0x5c, 0xa6, 0xfa];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    // The encoding follows the ABI specs defined  [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md)
    #[cfg(experimental)]
    #[test]
    fn enums_are_sized_to_fit_the_biggest_variant() -> Result<()> {
        // Our enum has two variants: B256, and U64. So the enum will set aside
        // 256b of space or 4 WORDS because that is the space needed to fit the
        // largest variant(B256).
        let types = to_named(&[ParamType::B256, ParamType::U64]);
        let enum_variants = EnumVariants::new(types)?;
        let enum_selector = Box::new((1, Token::U64(42), enum_variants));

        let encoded = ABIEncoder::default()
            .encode(slice::from_ref(&Token::Enum(enum_selector)))?
            .resolve(0);

        let enum_discriminant_enc = vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];
        let u64_enc = vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a];
        let enum_padding = vec![0x0; 24];

        // notice the ordering, first the discriminant, then the necessary
        // padding and then the value itself.
        let expected: Vec<u8> = [enum_discriminant_enc, enum_padding, u64_enc]
            .into_iter()
            .flatten()
            .collect();

        assert_eq!(hex::encode(expected), hex::encode(encoded));
        Ok(())
    }

    #[test]
    fn encoding_enums_with_deeply_nested_types() -> Result<()> {
        /*
        enum DeeperEnum {
            v1: bool,
            v2: str[10]
        }
         */
        let types = to_named(&[ParamType::Bool, ParamType::StringArray(10)]);
        let deeper_enum_variants = EnumVariants::new(types)?;
        let deeper_enum_token =
            Token::StringArray(StaticStringToken::new("0123456789".into(), Some(10)));

        /*
        struct StructA {
            some_enum: DeeperEnum
            some_number: u32
        }
         */

        let fields = to_named(&[
            ParamType::Enum {
                name: "".to_string(),
                enum_variants: deeper_enum_variants.clone(),
                generics: vec![],
            },
            ParamType::Bool,
        ]);
        let struct_a_type = ParamType::Struct {
            name: "".to_string(),
            fields,
            generics: vec![],
        };

        let struct_a_token = Token::Struct(vec![
            Token::Enum(Box::new((1, deeper_enum_token, deeper_enum_variants))),
            Token::U32(11332),
        ]);

        /*
         enum TopLevelEnum {
            v1: StructA,
            v2: bool,
            v3: u64
        }
        */

        let types = to_named(&[struct_a_type, ParamType::Bool, ParamType::U64]);
        let top_level_enum_variants = EnumVariants::new(types)?;
        let top_level_enum_token =
            Token::Enum(Box::new((0, struct_a_token, top_level_enum_variants)));

        let result = ABIEncoder::default()
            .encode(slice::from_ref(&top_level_enum_token))?
            .resolve(0);

        #[cfg(experimental)]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, // TopLevelEnum::v1 discriminant
            0, 0, 0, 0, 0, 0, 0, 1, // DeeperEnum::v2 discriminant
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, // str[10]
            0, 0, 0, 0, 0, 0, // DeeperEnum padding
            0, 0, 0, 0, 0, 0, 44, 68, // StructA.some_number
        ];
        #[cfg(not(experimental))]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, // TopLevelEnum::v1 discriminant
            0, 0, 0, 0, 0, 0, 0, 1, // DeeperEnum::v2 discriminant
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, // str[10]
            0, 0, 44, 68, // StructA.some_number
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_function_with_nested_structs() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type":"function",
        //         "inputs": [{"name":"arg","type":"Foo"}],
        //         "name":"takes_my_nested_struct",
        //         "outputs": []
        //     }
        // ]
        // "#;

        // struct Foo {
        //     x: u16,
        //     y: Bar,
        // }
        //
        // struct Bar {
        //     a: bool,
        //     b: u8[2],
        // }

        let fn_signature = "takes_my_nested_struct(Foo)";

        let args: Vec<Token> = vec![Token::Struct(vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ])];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, // 10u16
            0x1, // true
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // padding
            0x1, 0x2, // [1u8, 2u8]
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // padding
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0xea, 0x0a, 0xfd, 0x23];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        println!("Encoded ABI for ({fn_signature}): {encoded:#0x?}");

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    #[cfg(experimental)]
    fn encode_comprehensive_function() -> Result<()> {
        // let json_abi =
        // r#"
        // [
        //     {
        //         "type": "contract",
        //         "inputs": [
        //         {
        //             "name": "arg",
        //             "type": "Foo"
        //         },
        //         {
        //             "name": "arg2",
        //             "type": "u8[2]"
        //         },
        //         {
        //             "name": "arg3",
        //             "type": "b256"
        //         },
        //         {
        //             "name": "arg",
        //             "type": "str[23]"
        //         }
        //         ],
        //         "name": "long_function",
        //         "outputs": []
        //     }
        // ]
        // "#;

        // struct Foo {
        //     x: u16,
        //     y: Bar,
        // }
        //
        // struct Bar {
        //     a: bool,
        //     b: u8[2],
        // }

        let fn_signature = "long_function(Foo,u8[2],b256,str[23])";

        let foo = Token::Struct(vec![
            Token::U16(10),
            Token::Struct(vec![
                Token::Bool(true),
                Token::Array(vec![Token::U8(1), Token::U8(2)]),
            ]),
        ]);

        let u8_arr = Token::Array(vec![Token::U8(1), Token::U8(2)]);

        let mut hasher = Sha256::new();
        hasher.update("test string".as_bytes());

        let b256 = Token::B256(hasher.finalize().into());

        let s = Token::StringArray(StaticStringToken::new(
            "This is a full sentence".into(),
            Some(23),
        ));

        let args: Vec<Token> = vec![foo, u8_arr, b256, s];

        let expected_encoded_abi = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, // foo.x == 10u16
            0x1, // foo.y.a == true
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // foo.y.a padding
            0x1, // foo.y.b.0 == 1u8
            0x2, // foo.y.b.1 == 2u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // foo.y.a
            0x1, // u8[2].0 == 1u8
            0x2, // u8[2].0 == 2u8
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xd5, 0x57, 0x9c, 0x46, 0xdf, 0xcc, 0x7f,
            0x18, // b256
            0x20, 0x70, 0x13, 0xe6, 0x5b, 0x44, 0xe4, 0xcb, // b256
            0x4e, 0x2c, 0x22, 0x98, 0xf4, 0xac, 0x45, 0x7b, // b256
            0xa8, 0xf8, 0x27, 0x43, 0xf3, 0x1e, 0x93, 0xb, // b256
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, // str[23]
            0x61, 0x20, 0x66, 0x75, 0x6c, 0x6c, 0x20, 0x73, // str[23]
            0x65, 0x6e, 0x74, 0x65, 0x6e, 0x63, 0x65, // str[23]
            0x0,
        ];

        let expected_function_selector = [0x0, 0x0, 0x0, 0x0, 0x10, 0x93, 0xb2, 0x12];

        let encoded_function_selector = first_four_bytes_of_sha256_hash(fn_signature);

        let encoded = ABIEncoder::default().encode(&args)?.resolve(0);

        assert_eq!(hex::encode(expected_encoded_abi), hex::encode(encoded));
        assert_eq!(encoded_function_selector, expected_function_selector);
        Ok(())
    }

    #[test]
    fn enums_with_only_unit_variants_are_encoded_in_one_word() -> Result<()> {
        let expected = [0, 0, 0, 0, 0, 0, 0, 1];

        let types = to_named(&[ParamType::Unit, ParamType::Unit]);
        let enum_selector = Box::new((1, Token::Unit, EnumVariants::new(types)?));

        let actual = ABIEncoder::default()
            .encode(&[Token::Enum(enum_selector)])?
            .resolve(0);

        assert_eq!(actual, expected);

        Ok(())
    }

    #[cfg(experimental)]
    #[test]
    fn units_in_composite_types_are_encoded_in_one_word() -> Result<()> {
        let expected = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5];

        let actual = ABIEncoder::default()
            .encode(&[Token::Struct(vec![Token::Unit, Token::U32(5)])])?
            .resolve(0);

        assert_eq!(actual, expected);
        Ok(())
    }

    #[cfg(experimental)]
    #[test]
    fn enums_with_units_are_correctly_padded() -> Result<()> {
        let discriminant = vec![0, 0, 0, 0, 0, 0, 0, 1];
        let padding = vec![0; 32];
        let expected: Vec<u8> = [discriminant, padding].into_iter().flatten().collect();

        let types = to_named(&[ParamType::B256, ParamType::Unit]);
        let enum_selector = Box::new((1, Token::Unit, EnumVariants::new(types)?));

        let actual = ABIEncoder::default()
            .encode(&[Token::Enum(enum_selector)])?
            .resolve(0);

        assert_eq!(actual, expected);
        Ok(())
    }

    #[cfg(experimental)]
    #[test]
    fn vector_has_ptr_cap_len_and_then_data() -> Result<()> {
        // arrange
        let offset: u8 = 150;
        let token = Token::Vector(vec![Token::U64(5)]);

        // act
        let result = ABIEncoder::default()
            .encode(&[token])?
            .resolve(offset as u64);

        // assert
        let ptr = [0, 0, 0, 0, 0, 0, 0, 3 * WORD_SIZE as u8 + offset];
        let cap = [0, 0, 0, 0, 0, 0, 0, 1];
        let len = [0, 0, 0, 0, 0, 0, 0, 1];
        let data = [0, 0, 0, 0, 0, 0, 0, 5];

        let expected = chain!(ptr, cap, len, data).collect::<Vec<_>>();

        assert_eq!(result, expected);

        Ok(())
    }

    #[cfg(experimental)]
    #[test]
    fn data_from_two_vectors_aggregated_at_the_end() -> Result<()> {
        // arrange
        let offset: u8 = 40;
        let vec_1 = Token::Vector(vec![Token::U64(5)]);
        let vec_2 = Token::Vector(vec![Token::U64(6)]);

        // act
        let result = ABIEncoder::default()
            .encode(&[vec_1, vec_2])?
            .resolve(offset as u64);

        // assert
        let vec1_data_offset = 6 * WORD_SIZE as u8 + offset;
        let vec1_ptr = [0, 0, 0, 0, 0, 0, 0, vec1_data_offset];
        let vec1_cap = [0, 0, 0, 0, 0, 0, 0, 1];
        let vec1_len = [0, 0, 0, 0, 0, 0, 0, 1];
        let vec1_data = [0, 0, 0, 0, 0, 0, 0, 5];

        let vec2_data_offset = vec1_data_offset + vec1_data.len() as u8;
        let vec2_ptr = [0, 0, 0, 0, 0, 0, 0, vec2_data_offset];
        let vec2_cap = [0, 0, 0, 0, 0, 0, 0, 1];
        let vec2_len = [0, 0, 0, 0, 0, 0, 0, 1];
        let vec2_data = [0, 0, 0, 0, 0, 0, 0, 6];

        let expected = chain!(
            vec1_ptr, vec1_cap, vec1_len, vec2_ptr, vec2_cap, vec2_len, vec1_data, vec2_data,
        )
        .collect::<Vec<_>>();

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn vec_in_enum() -> Result<()> {
        // arrange
        let offset = 40;
        let types = to_named(&[ParamType::B256, ParamType::Vector(Box::new(ParamType::U64))]);
        let variants = EnumVariants::new(types)?;
        let selector = (1, Token::Vector(vec![Token::U64(5)]), variants);
        let token = Token::Enum(Box::new(selector));

        // act
        let result = ABIEncoder::default()
            .encode(&[token])?
            .resolve(offset as u64);

        // assert
        #[cfg(experimental)]
        let expected = {
            let discriminant = vec![0, 0, 0, 0, 0, 0, 0, 1];

            const PADDING: usize = std::mem::size_of::<[u8; 32]>() - VEC_METADATA_SIZE;

            let vec1_ptr = ((DISCRIMINANT_SIZE + PADDING + VEC_METADATA_SIZE + offset) as u64)
                .to_be_bytes()
                .to_vec();
            let vec1_cap = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_len = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_data = [0, 0, 0, 0, 0, 0, 0, 5];

            chain!(
                discriminant,
                vec![0; PADDING],
                vec1_ptr,
                vec1_cap,
                vec1_len,
                vec1_data
            )
            .collect::<Vec<u8>>()
        };
        #[cfg(not(experimental))]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 1, // enum dicsriminant
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5, // vec[len, u64]
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn enum_in_vec() -> Result<()> {
        // arrange
        let offset = 40;
        let types = to_named(&[ParamType::B256, ParamType::U8]);
        let variants = EnumVariants::new(types)?;
        let selector = (1, Token::U8(8), variants);
        let enum_token = Token::Enum(Box::new(selector));

        let vec_token = Token::Vector(vec![enum_token]);

        // act
        let result = ABIEncoder::default()
            .encode(&[vec_token])?
            .resolve(offset as u64);

        // assert
        #[cfg(experimental)]
        let expected = {
            const PADDING: usize = std::mem::size_of::<[u8; 32]>() - WORD_SIZE;

            let vec1_ptr = ((VEC_METADATA_SIZE + offset) as u64).to_be_bytes().to_vec();
            let vec1_cap = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_len = [0, 0, 0, 0, 0, 0, 0, 1];
            let discriminant = 1u64.to_be_bytes();
            let vec1_data =
                chain!(discriminant, [0; PADDING], 8u64.to_be_bytes()).collect::<Vec<_>>();

            chain!(vec1_ptr, vec1_cap, vec1_len, vec1_data).collect::<Vec<u8>>()
        };
        #[cfg(not(experimental))]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 1, // vec len
            0, 0, 0, 0, 0, 0, 0, 1, 8, // enum discriminant and u8 value
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn vec_in_struct() -> Result<()> {
        // arrange
        let offset = 40;
        let token = Token::Struct(vec![Token::Vector(vec![Token::U64(5)]), Token::U8(9)]);

        // act
        let result = ABIEncoder::default()
            .encode(&[token])?
            .resolve(offset as u64);

        // assert
        #[cfg(experimental)]
        let expected = {
            let vec1_ptr = ((VEC_METADATA_SIZE + WORD_SIZE + offset) as u64)
                .to_be_bytes()
                .to_vec();
            let vec1_cap = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_len = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_data = [0, 0, 0, 0, 0, 0, 0, 5];

            chain!(vec1_ptr, vec1_cap, vec1_len, [9], [0; 7], vec1_data).collect::<Vec<u8>>()
        };
        #[cfg(not(experimental))]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5, // vec[len, u64]
            9, // u8
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn vec_in_vec() -> Result<()> {
        // arrange
        let offset = 40;
        let token = Token::Vector(vec![Token::Vector(vec![Token::U8(5), Token::U8(6)])]);

        // act
        let result = ABIEncoder::default()
            .encode(&[token])?
            .resolve(offset as u64);

        // assert
        #[cfg(experimental)]
        let expected = {
            let vec1_data_offset = (VEC_METADATA_SIZE + offset) as u64;
            let vec1_ptr = vec1_data_offset.to_be_bytes().to_vec();
            let vec1_cap = [0, 0, 0, 0, 0, 0, 0, 1];
            let vec1_len = [0, 0, 0, 0, 0, 0, 0, 1];

            let vec2_ptr = (vec1_data_offset + VEC_METADATA_SIZE as u64)
                .to_be_bytes()
                .to_vec();
            let vec2_cap = [0, 0, 0, 0, 0, 0, 0, 2];
            let vec2_len = [0, 0, 0, 0, 0, 0, 0, 2];
            let vec2_data = [5, 6];

            let vec1_data = chain!(vec2_ptr, vec2_cap, vec2_len, vec2_data).collect::<Vec<_>>();

            chain!(vec1_ptr, vec1_cap, vec1_len, vec1_data).collect::<Vec<u8>>()
        };
        #[cfg(not(experimental))]
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 1, // vec1 len
            0, 0, 0, 0, 0, 0, 0, 2, 5, 6, // vec2 [len, u8, u8]
        ];

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn encoding_bytes() -> Result<()> {
        // arrange
        let token = Token::Bytes(vec![1, 2, 3]);
        let offset = 40;

        // act
        let result = ABIEncoder::default().encode(&[token])?.resolve(offset);

        // assert
        #[cfg(experimental)]
        let expected = {
            let ptr = [0, 0, 0, 0, 0, 0, 0, 64];
            let cap = [0, 0, 0, 0, 0, 0, 0, 8];
            let len = [0, 0, 0, 0, 0, 0, 0, 3];
            let data = [1, 2, 3, 0, 0, 0, 0, 0];

            [ptr, cap, len, data].concat()
        };
        #[cfg(not(experimental))]
        let expected = [0, 0, 0, 0, 0, 0, 0, 3, 1, 2, 3]; // bytes[len, u8, u8, u8]

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn encoding_raw_slices() -> Result<()> {
        // arrange
        let token = Token::RawSlice(vec![1, 2, 3]);
        let offset = 40;

        // act
        let result = ABIEncoder::default().encode(&[token])?.resolve(offset);

        // assert
        #[cfg(experimental)]
        let expected = {
            let ptr = [0, 0, 0, 0, 0, 0, 0, 56].to_vec();
            let len = [0, 0, 0, 0, 0, 0, 0, 3].to_vec();
            let data = [1, 2, 3].to_vec();
            let padding = [0, 0, 0, 0, 0].to_vec();

            [ptr, len, data, padding].concat()
        };
        #[cfg(not(experimental))]
        let expected = [0, 0, 0, 0, 0, 0, 0, 3, 1, 2, 3]; // raw_slice[len, u8, u8, u8]

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn encoding_std_string() -> Result<()> {
        // arrange
        let string = String::from("This ");
        let token = Token::String(string);
        let offset = 40;

        // act
        let result = ABIEncoder::default().encode(&[token])?.resolve(offset);

        // assert
        #[cfg(experimental)]
        let expected = {
            let ptr = [0, 0, 0, 0, 0, 0, 0, 64];
            let cap = [0, 0, 0, 0, 0, 0, 0, 8];
            let len = [0, 0, 0, 0, 0, 0, 0, 5];
            let data = [0x54, 0x68, 0x69, 0x73, 0x20, 0, 0, 0];

            [ptr, cap, len, data].concat()
        };
        #[cfg(not(experimental))]
        let expected = [0, 0, 0, 0, 0, 0, 0, 5, 84, 104, 105, 115, 32]; // string[len, data]

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn encoding_large_unsigned_integers() -> Result<()> {
        {
            let token = Token::U128(u128::MAX);
            let expected_encoding = [255; 16];

            let result = ABIEncoder::default().encode(&[token])?.resolve(0);

            assert_eq!(result, expected_encoding);
        }
        {
            let token = Token::U256(U256::MAX);
            let expected_encoding = [255; 32];

            let result = ABIEncoder::default().encode(&[token])?.resolve(0);

            assert_eq!(result, expected_encoding);
        }

        Ok(())
    }

    #[cfg(experimental)]
    #[test]
    fn capacity_overflow_is_caught() -> Result<()> {
        let token = Token::Enum(Box::new((
            1,
            Token::String("".to_string()),
            EnumVariants::new(to_named(&[
                ParamType::StringArray(18446742977385549567),
                ParamType::U8,
            ]))?,
        )));
        let capacity_overflow_error = ABIEncoder::default().encode(&[token]).unwrap_err();

        assert!(capacity_overflow_error
            .to_string()
            .contains("Try increasing maximum total enum width"));

        Ok(())
    }

    #[test]
    fn max_depth_surpassed() {
        const MAX_DEPTH: usize = 2;
        let config = EncoderConfig {
            max_depth: MAX_DEPTH,
            ..Default::default()
        };
        let msg = "depth limit `2` reached while encoding. Try increasing it".to_string();

        [nested_struct, nested_enum, nested_tuple, nested_array]
            .iter()
            .map(|fun| fun(MAX_DEPTH + 1))
            .for_each(|token| {
                assert_encoding_failed(config, token, &msg);
            });
    }

    #[test]
    fn encoder_for_configurables_optimizes_top_level_u8() {
        // given
        let encoder = ConfigurablesEncoder::default();

        // when
        let encoded = encoder.encode(&[Token::U8(255)]).unwrap().resolve(0);

        // then
        assert_eq!(encoded, vec![255]);
    }

    #[test]
    fn encoder_for_configurables_optimizes_top_level_bool() {
        // given
        let encoder = ConfigurablesEncoder::default();

        // when
        let encoded = encoder.encode(&[Token::Bool(true)]).unwrap().resolve(0);

        // then
        assert_eq!(encoded, vec![1]);
    }

    fn assert_encoding_failed(config: EncoderConfig, token: Token, msg: &str) {
        let encoder = ABIEncoder::new(config);

        let err = encoder.encode(&[token]);

        let Err(Error::Codec(actual_msg)) = err else {
            panic!("expected a Codec error. Got: `{err:?}`");
        };
        assert_eq!(actual_msg, msg);
    }

    fn nested_struct(depth: usize) -> Token {
        let fields = if depth == 1 {
            vec![Token::U8(255), Token::String("bloopblip".to_string())]
        } else {
            vec![nested_struct(depth - 1)]
        };

        Token::Struct(fields)
    }

    fn nested_enum(depth: usize) -> Token {
        if depth == 0 {
            return Token::U8(255);
        }

        let inner_enum = nested_enum(depth - 1);

        // Create a basic EnumSelector for the current level (the `EnumVariants` is not
        // actually accurate but it's not used for encoding)
        let selector = (
            0u64,
            inner_enum,
            EnumVariants::new(to_named(&[ParamType::U64])).unwrap(),
        );

        Token::Enum(Box::new(selector))
    }

    fn nested_array(depth: usize) -> Token {
        if depth == 1 {
            Token::Array(vec![Token::U8(255)])
        } else {
            Token::Array(vec![nested_array(depth - 1)])
        }
    }

    fn nested_tuple(depth: usize) -> Token {
        let fields = if depth == 1 {
            vec![Token::U8(255), Token::String("bloopblip".to_string())]
        } else {
            vec![nested_tuple(depth - 1)]
        };

        Token::Tuple(fields)
    }
}
