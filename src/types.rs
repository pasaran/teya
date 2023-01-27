type TypeId = usize;

enum TypeKind {
    None,
    Struct( TypeStruct ),
    Enum( TypeEnum ),
    Tuple( TypeTuple ),
    Array( TypeArray ),
    Fn( TypeFn ),
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct TypeStruct {
    fields: Vec< TypeStructField >,
}

struct TypeStructField {
    name: String,
    type_id: TypeId,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct TypeEnum {
    fields: TypeEnumField,
}

enum TypeEnumFieldKind {
    None,
    Tuple,
    Record,
}

struct TypeEnumField {
    name: String,
    kind: TypeEnumFieldKind,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct TypeTuple {
    size: usize,
    item_type_ids: Vec< TypeId >,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct TypeArray {
    size: usize,
    item_type_id: TypeId,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct TypeFn {
    params: Vec< TypeFnParam >,
    return_type_id: TypeId,
}

struct TypeFnParam {
    name: String,
    type_id: TypeId,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct Type {
    name: String,
    generic_params: Vec< TypeGenericParam >,
    kind: TypeKind,
}

struct TypeGenericParam {
    name: String,
}
