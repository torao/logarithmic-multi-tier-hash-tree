// This file is generated by rust-protobuf 2.22.1. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `src/proto.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_22_1;

#[derive(PartialEq,Clone,Default)]
pub struct Intermediate {
    // message fields
    pub left_node: u64,
    pub right_node: u64,
    pub hash: ::std::vec::Vec<u8>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Intermediate {
    fn default() -> &'a Intermediate {
        <Intermediate as ::protobuf::Message>::default_instance()
    }
}

impl Intermediate {
    pub fn new() -> Intermediate {
        ::std::default::Default::default()
    }

    // uint64 left_node = 1;


    pub fn get_left_node(&self) -> u64 {
        self.left_node
    }
    pub fn clear_left_node(&mut self) {
        self.left_node = 0;
    }

    // Param is passed by value, moved
    pub fn set_left_node(&mut self, v: u64) {
        self.left_node = v;
    }

    // uint64 right_node = 2;


    pub fn get_right_node(&self) -> u64 {
        self.right_node
    }
    pub fn clear_right_node(&mut self) {
        self.right_node = 0;
    }

    // Param is passed by value, moved
    pub fn set_right_node(&mut self, v: u64) {
        self.right_node = v;
    }

    // bytes hash = 3;


    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }
    pub fn clear_hash(&mut self) {
        self.hash.clear();
    }

    // Param is passed by value, moved
    pub fn set_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.hash = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.hash
    }

    // Take field
    pub fn take_hash(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.hash, ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for Intermediate {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.left_node = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.right_node = tmp;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.hash)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.left_node != 0 {
            my_size += ::protobuf::rt::value_size(1, self.left_node, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.right_node != 0 {
            my_size += ::protobuf::rt::value_size(2, self.right_node, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.hash.is_empty() {
            my_size += ::protobuf::rt::bytes_size(3, &self.hash);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.left_node != 0 {
            os.write_uint64(1, self.left_node)?;
        }
        if self.right_node != 0 {
            os.write_uint64(2, self.right_node)?;
        }
        if !self.hash.is_empty() {
            os.write_bytes(3, &self.hash)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Intermediate {
        Intermediate::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "left_node",
                |m: &Intermediate| { &m.left_node },
                |m: &mut Intermediate| { &mut m.left_node },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "right_node",
                |m: &Intermediate| { &m.right_node },
                |m: &mut Intermediate| { &mut m.right_node },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "hash",
                |m: &Intermediate| { &m.hash },
                |m: &mut Intermediate| { &mut m.hash },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Intermediate>(
                "Intermediate",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Intermediate {
        static instance: ::protobuf::rt::LazyV2<Intermediate> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Intermediate::new)
    }
}

impl ::protobuf::Clear for Intermediate {
    fn clear(&mut self) {
        self.left_node = 0;
        self.right_node = 0;
        self.hash.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Intermediate {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Intermediate {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Leaf {
    // message fields
    pub payload: ::std::vec::Vec<u8>,
    pub hash: ::std::vec::Vec<u8>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Leaf {
    fn default() -> &'a Leaf {
        <Leaf as ::protobuf::Message>::default_instance()
    }
}

impl Leaf {
    pub fn new() -> Leaf {
        ::std::default::Default::default()
    }

    // bytes payload = 1;


    pub fn get_payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::vec::Vec<u8>) {
        self.payload = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.payload
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.payload, ::std::vec::Vec::new())
    }

    // bytes hash = 2;


    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }
    pub fn clear_hash(&mut self) {
        self.hash.clear();
    }

    // Param is passed by value, moved
    pub fn set_hash(&mut self, v: ::std::vec::Vec<u8>) {
        self.hash = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hash(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.hash
    }

    // Take field
    pub fn take_hash(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.hash, ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for Leaf {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.payload)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.hash)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.payload.is_empty() {
            my_size += ::protobuf::rt::bytes_size(1, &self.payload);
        }
        if !self.hash.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.hash);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.payload.is_empty() {
            os.write_bytes(1, &self.payload)?;
        }
        if !self.hash.is_empty() {
            os.write_bytes(2, &self.hash)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Leaf {
        Leaf::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "payload",
                |m: &Leaf| { &m.payload },
                |m: &mut Leaf| { &mut m.payload },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "hash",
                |m: &Leaf| { &m.hash },
                |m: &mut Leaf| { &mut m.hash },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Leaf>(
                "Leaf",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Leaf {
        static instance: ::protobuf::rt::LazyV2<Leaf> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Leaf::new)
    }
}

impl ::protobuf::Clear for Leaf {
    fn clear(&mut self) {
        self.payload.clear();
        self.hash.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Leaf {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Leaf {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0fsrc/proto.proto\x12\x07lshtree\"^\n\x0cIntermediate\x12\x1b\n\tlef\
    t_node\x18\x01\x20\x01(\x04R\x08leftNode\x12\x1d\n\nright_node\x18\x02\
    \x20\x01(\x04R\trightNode\x12\x12\n\x04hash\x18\x03\x20\x01(\x0cR\x04has\
    h\"4\n\x04Leaf\x12\x18\n\x07payload\x18\x01\x20\x01(\x0cR\x07payload\x12\
    \x12\n\x04hash\x18\x02\x20\x01(\x0cR\x04hashJ\xdf\x02\n\x06\x12\x04\0\0\
    \x0c\x01\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\x08\n\x01\x02\x12\x03\x01\0\
    \x10\n\n\n\x02\x04\0\x12\x04\x03\0\x07\x01\n\n\n\x03\x04\0\x01\x12\x03\
    \x03\x08\x14\n\x0b\n\x04\x04\0\x02\0\x12\x03\x04\x02\x17\n\x0c\n\x05\x04\
    \0\x02\0\x05\x12\x03\x04\x02\x08\n\x0c\n\x05\x04\0\x02\0\x01\x12\x03\x04\
    \t\x12\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\x04\x15\x16\n\x0b\n\x04\x04\0\
    \x02\x01\x12\x03\x05\x02\x18\n\x0c\n\x05\x04\0\x02\x01\x05\x12\x03\x05\
    \x02\x08\n\x0c\n\x05\x04\0\x02\x01\x01\x12\x03\x05\t\x13\n\x0c\n\x05\x04\
    \0\x02\x01\x03\x12\x03\x05\x16\x17\n\x0b\n\x04\x04\0\x02\x02\x12\x03\x06\
    \x02\x11\n\x0c\n\x05\x04\0\x02\x02\x05\x12\x03\x06\x02\x07\n\x0c\n\x05\
    \x04\0\x02\x02\x01\x12\x03\x06\x08\x0c\n\x0c\n\x05\x04\0\x02\x02\x03\x12\
    \x03\x06\x0f\x10\n\n\n\x02\x04\x01\x12\x04\t\0\x0c\x01\n\n\n\x03\x04\x01\
    \x01\x12\x03\t\x08\x0c\n\x0b\n\x04\x04\x01\x02\0\x12\x03\n\x02\x14\n\x0c\
    \n\x05\x04\x01\x02\0\x05\x12\x03\n\x02\x07\n\x0c\n\x05\x04\x01\x02\0\x01\
    \x12\x03\n\x08\x0f\n\x0c\n\x05\x04\x01\x02\0\x03\x12\x03\n\x12\x13\n\x0b\
    \n\x04\x04\x01\x02\x01\x12\x03\x0b\x02\x11\n\x0c\n\x05\x04\x01\x02\x01\
    \x05\x12\x03\x0b\x02\x07\n\x0c\n\x05\x04\x01\x02\x01\x01\x12\x03\x0b\x08\
    \x0c\n\x0c\n\x05\x04\x01\x02\x01\x03\x12\x03\x0b\x0f\x10b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
