use crate::string::JString;

#[derive(Clone, Debug)]
pub enum Constant {
    Utf8(JString),
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    LongOrDoubleHigh,
    Class(u16),
    String(u16),
    FieldRef {
        class: u16,
        name_and_type: u16,
    },
    MethodRef {
        class: u16,
        name_and_type: u16,
    },
    InterfaceMethodRef {
        class: u16,
        name_and_type: u16,
    },
    NameAndType {
        name: u16,
        descriptor: u16,
    },
    MethodHandle {
        kind: u8,
        reference: u16,
    },
    MethodType(u16),
    Dynamic {
        bootstrap_attrs: u16,
        name_and_type: u16,
    },
    InvokeDynamic {
        bootstrap_attrs: u16,
        name_and_type: u16,
    },
    Module(u16),
    Package(u16),
}

#[derive(Clone, Debug)]
pub struct ClassFile {
    pub min: u16,
    pub maj: u16,
    pub consts: Vec<Constant>,
    pub acc: u16,
    pub this: u16,
    pub supercl: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>,
}

pub mod consts {
    pub const MAGIC: u32 = 0xCAFEBABE;
    pub const MIN_VERSION: u16 = 45;
    pub const MAX_VERSION: u16 = 60;
    pub const PREVIEW_FEATURES: u16 = 0xffff;

    pub const ACC_CLASS_BITS: u16 = ACC_PUBLIC
        | ACC_FINAL
        | ACC_SUPER
        | ACC_INTERFACE
        | ACC_ABSTRACT
        | ACC_SYNTHETIC
        | ACC_ANNOTATION
        | ACC_ENUM
        | ACC_MODULE;

    pub const ACC_FIELD_BITS: u16 = ACC_PUBLIC
        | ACC_PRIVATE
        | ACC_PROTECTED
        | ACC_STATIC
        | ACC_FINAL
        | ACC_VOLATILE
        | ACC_TRANSIENT
        | ACC_SYNTHETIC
        | ACC_ENUM;

    pub const ACC_METHOD_BITS: u16 = ACC_PUBLIC
        | ACC_PRIVATE
        | ACC_PROTECTED
        | ACC_STATIC
        | ACC_FINAL
        | ACC_SYNCHRONIZED
        | ACC_BRIDGE
        | ACC_VARARGS
        | ACC_NATIVE
        | ACC_ABSTRACT
        | ACC_STRICT
        | ACC_SYNTHETIC;

    pub const ACC_INNER_CLASS_BITS: u16 =
        ACC_CLASS_BITS & !ACC_MODULE | ACC_PROTECTED | ACC_PRIVATE | ACC_STATIC;

    pub const ACC_REQUIRES_BITS: u16 =
        ACC_TRANSITIVE | ACC_STATIC_PHASE | ACC_SYNTHETIC | ACC_MANDATED;
    pub const ACC_EXPORTS_BITS: u16 = ACC_SYNTHETIC | ACC_MANDATED;
    pub const ACC_PARAMETER_BITS: u16 = ACC_FINAL | ACC_SYNTHETIC | ACC_MANDATED;

    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_SUPER: u16 = 0x0020;
    pub const ACC_TRANSITIVE: u16 = 0x0020;
    pub const ACC_SYNCHRONIZED: u16 = 0x0020;
    pub const ACC_VOLATILE: u16 = 0x0040;
    pub const ACC_STATIC_PHASE: u16 = 0x0040;
    pub const ACC_BRIDGE: u16 = 0x0040;
    pub const ACC_TRANSIENT: u16 = 0x0080;
    pub const ACC_VARARGS: u16 = 0x0080;
    pub const ACC_NATIVE: u16 = 0x0100;
    pub const ACC_INTERFACE: u16 = 0x0200;
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_STRICT: u16 = 0x0800;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ANNOTATION: u16 = 0x2000;
    pub const ACC_ENUM: u16 = 0x4000;
    pub const ACC_MODULE: u16 = 0x8000;
    pub const ACC_MANDATED: u16 = 0x8000;
}

#[derive(Clone, Debug)]
pub enum Attribute {
    ConstantValue(u16),
    Code(CodeAttribute),
    StackMapTable(Vec<StackMapFrame>),
    Exceptions(Vec<u16>),
    InnerClasses(Vec<InnerClassInfo>),
    EnclosingMethod { class: u16, method: u16 },
    Synthetic,
    Signature(u16),
    SourceFile(u16),
    SourceDebugExtension(JString),
    LineNumberTable(Vec<LineNumberEntry>),
    LocalVariableTable(Vec<LocalVariableInfo>),
    LocalVariableTypeTable(Vec<LocalVariableTypeInfo>),
    Deprecated,
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeInvisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeVisibleTypeAnnotations(Vec<TypeAnnotation>),
    RuntimeInvisibleTypeAnnotations(Vec<TypeAnnotation>),
    AnnotationDefault(ElementValue),
    BootstrapMethods(Vec<BootstrapMethod>),
    MethodParameters(Vec<ParameterInfo>),
    Module(ModuleInfo),
    ModulePackage(Vec<u16>),
    ModuleMainClass(u16),
    NestHost(u16),
    NestMembers(Vec<u16>),
    Record(Vec<RecordComponentInfo>),
    PermittedSubclasses(Vec<u16>),
    Availability(Availability),
    LangItem(u16),
    FillNativeMethod(u16),
    Unresolved { name: u16, content: Vec<u8> },
}

#[derive(Clone, Debug)]
pub enum Availability {
    From { ver: u16, default: bool },
    Removed { ver: u16, default: bool },
    Unstable { feature: u16, default: bool },
}

#[derive(Clone, Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exceptions: Vec<ExceptionInfo>,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct ExceptionInfo {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Clone, Debug)]
pub enum StackMapFrame {
    Same,
    SameLocals1StackFrame(VerificationInfo),
    SameLocals1StackFrameExtended {
        offset_deleta: u16,
        info: VerificationInfo,
    },
    ChopFrame {
        chop: u8,
        offset_delta: u16,
    },
    SameExtended {
        offset_delta: u16,
    },
    Append {
        offset_delta: u16,
        items: Vec<VerificationInfo>,
    },
    Full {
        offset_delta: u16,
        locals: Vec<VerificationInfo>,
        stack: Vec<VerificationInfo>,
    },
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum VerificationInfo {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,
    UninitializedThis,
    Object { class: u16 },
    Uninitialized { offset: u16 },
}

#[derive(Clone, Debug)]
pub struct InnerClassInfo {
    pub inner_class_info: u16,
    pub outer_class_info: u16,
    pub inner_name: u16,
    pub inner_flags: u16,
}

#[derive(Clone, Debug)]
pub struct LineNumberEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Clone, Debug)]
pub struct LocalVariableInfo {
    pub start_pc: u16,
    pub length: u16,
    pub name: u16,
    pub descriptor: u16,
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct LocalVariableLocationInfo {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct LocalVariableTypeInfo {
    pub start_pc: u16,
    pub length: u16,
    pub name: u16,
    pub signature: u16,
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub class: u16,
    pub elements: Vec<AnnotationElement>,
}

#[derive(Clone, Debug)]
pub struct AnnotationElement {
    pub name: u16,
    pub value: ElementValue,
}

#[derive(Clone, Debug)]
pub enum ElementValue {
    Byte(u16),
    Char(u16),
    Double(u16),
    Float(u16),
    Int(u16),
    Long(u16),
    Short(u16),
    Boolean(u16),
    String(u16),
    Enum { type_name: u16, const_name: u16 },
    Class(u16),
    Annotation(Annotation),
    Array(Vec<ElementValue>),
}

#[derive(Clone, Debug)]
pub struct TypeAnnotation {
    pub target: TypeAnnotationTarget,
    pub path: Vec<TypePathSegment>,
    pub annotation: Annotation,
}

#[derive(Clone, Debug)]
pub enum TypeAnnotationTarget {
    ClassTypeParameter(u8),
    MethodTypeParameter(u8),
    SuperClass(u16),
    ClassTypeParameterBound { param: u8, bound: u8 },
    MethodTypeParameterBound { param: u8, bound: u8 },
    FieldType,
    MethodReturnType,
    RecieverType,
    FormalParameterType(u8),
    ThrowsType(u16),
    LocalVariableType(Vec<LocalVariableLocationInfo>),
    ResourceVariableType(Vec<LocalVariableLocationInfo>),
    CatchParameterType(u16),
    InstanceOfType(u16),
    NewType(u16),
    NewReferenceType(u16),
    MethodReferenceType(u16),
    CastType { offset: u16, type_var: u8 },
    GenericConstructorTypeArgument { offset: u16, type_var: u8 },
    GenericMethodTypeArgument { offset: u16, type_var: u8 },
    GenericConstructorReferenceTypeArgument { offset: u16, type_var: u8 },
    GenericMethodReferenceTypeArgument { offset: u16, type_var: u8 },
}

#[derive(Clone, Debug)]
pub enum TypePathSegment {
    Array,
    NestedType,
    Wildcard,
    ParameterizedType(u8),
}

#[derive(Clone, Debug)]
pub struct BootstrapMethod {
    pub href: u16,
    pub args: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct ParameterInfo {
    pub name: u16,
    pub access: u16,
}

#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub name: u16,
    pub access: u16,
    pub version: u16,
    pub requires: Vec<RequireInfo>,
    pub exports: Vec<ExportInfo>,
    pub opens: Vec<ExportInfo>,
    pub uses: Vec<u16>,
    pub provides: Vec<ProvidesInfo>,
}

#[derive(Clone, Debug)]
pub struct RequireInfo {
    pub requires: u16,
    pub flags: u16,
    pub version: u16,
}

#[derive(Clone, Debug)]
pub struct ExportInfo {
    pub exports: u16,
    pub flags: u16,
    pub to: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct ProvidesInfo {
    pub provides: u16,
    pub with: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct RecordComponentInfo {
    pub name: u16,
    pub descriptor: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub acc: u16,
    pub name: u16,
    pub descriptor: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct MethodInfo {
    pub acc: u16,
    pub name: u16,
    pub descriptor: u16,
    pub attributes: Vec<Attribute>,
}
