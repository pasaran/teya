var no = require( 'nommon' );

var R = require( './runtime.js' );

//  ---------------------------------------------------------------------------------------------------------------  //

var teya = require( './teya.js' );
require( './types.js' );

var types = teya.types;
var TYPE = teya.types;

var entities = require( './entities.json' );

var internal_funcs = require( './internal_funcs.js' );

//  ---------------------------------------------------------------------------------------------------------------  //

function deentitify( s ) {
    return s
        .replace( /&#(\d+);?/g, function ( _, code ) {
            return String.fromCharCode( code );
        } )
        .replace( /&#[xX]([A-Fa-f0-9]+);?/g, function ( _, hex ) {
            return String.fromCharCode( parseInt( hex, 16 ) );
        } )
        .replace( /&(\w+);/g, function ( entity, name ) {
            return entities[ name ] || entity;
        } );
}


//  # ASTs
//  ---------------------------------------------------------------------------------------------------------------  //

var asts = teya.asts = {};

//  Здесь можно добавить что-нибудь в базовый класс.
//  Например, так делают некоторые шаблоны, генерящие код.
//
asts.ast = {};


//  ## Items
//  ---------------------------------------------------------------------------------------------------------------  //

asts.items = {};

asts.items._init = function() {
    this.items = [];
};

asts.items.add = function( item ) {
    this.items.push( item );
};

asts.items.is_empty = function() {
    return ( this.items.length === 0 );
};

asts.items.length = function() {
    return this.items.length;
};

asts.items.at = function( index, value ) {
    var items = this.items;

    if ( value !== undefined ) {
        items[ index ] = value;

    } else {
        return items[ index ];
    }
};

asts.items.for_each = function( callback ) {
    var items = this.items;

    for ( var i = 0, l = items.length; i < l; i++ ) {
        callback( items[ i ], i );
    }
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items.starts_with_attr = function() {
    var items = this.items;

    if ( items.length === 0 ) {
        return false;
    }

    return items[ 0 ].starts_with_attr();
};

asts.items.closes_attrs = function() {
    var items = this.items;

    var R = false;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        //  FIXME: А тут не closes_attrs должно быть?!
        var r = items[ i ].closes_attrs();

        if ( r === true ) {
            return true;
        } else if ( r === undefined ) {
            R = undefined;
        }
    }

    return R;
};

asts.items.close_attrs = function( is_open, should_close ) {
    var type = this.get_type();
    if ( !( type === TYPE.XML || type === TYPE.ATTR ) ) {
        return;
    }

    var items = this.items;

    var _items = [];
    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];
        var type = item.get_type();

        if ( is_open ) {
            if ( type !== TYPE.ATTR ) {
                if ( item.starts_with_attr() === false ) {
                    _items.push( this.child( 'close_attrs' ) );

                    is_open = false;
                }

                item.close_attrs( is_open );
            }
            //  FIXME: Неплохо бы делать warning, если мы очевидно лишние выражения
            //  с типом `attr` выкинули.

            _items.push( item );
        } else {
            if ( type !== TYPE.ATTR ) {
                item.close_attrs( false );

                _items.push( item );
            }
        }
    }

    if ( should_close && is_open ) {
        _items.push( this.child( 'close_attrs' ) );
    }

    this.items = _items;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items.cast = function( to ) {
    to = to || this.get_type();
    //  console.log( 'ITEMS.CAST', this.id, this.get_type(), to );

    var items = this.items;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        var from = item.get_type();
        if ( from === TYPE.NONE ) {
            continue;
        }

        if ( !types.is_castable( from, to ) ) {
            item.error( 'CAST_ERROR', {
                from: from,
                to: to
            } );
        }

        item.cast( to );
    }
};

/*
asts.items.oncast = function( to ) {
    var items = this.items;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        items[ i ].cast( to );
    }
};
*/

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items.string_children = function() {
    var r = [];

    var items = this.items;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        r.push( items[ i ].to_string() );
    }

    return ( r.length ) ? r.join( '\n' ).replace( /^/gm, '    ' ) : '';
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items._get_type = function() {
    var items = this.items;
    var l = items.length;

    if ( l === 0 ) { return TYPE.STRING; }

    var prev = items[ 0 ];
    var prev_type = prev.get_type();

    for ( var i = 1; i < l; i++ ) {
        var item = items[ i ];
        var type = item.get_type();

        var common_type = types.join_types( prev_type, type );
        if ( common_type === TYPE.UNDEF ) {
            item.error( 'WRONG_TYPES', {
                type_a: prev_type,
                id_a: prev.id,
                type_b: type,
                id_b: item.id
            } );
        }

        prev = item;
        prev_type = common_type;
    }

    return prev_type;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items.apply = function( callback, params ) {
    var items = this.items;

    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        callback( item, params );
    }
};

asts.items.dowalk = function( callback, params, pkey, pvalue ) {
    callback( this, params, pkey, pvalue );

    var items = this.items;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        item.dowalk( callback, params, i, this );
    }
};

asts.items.walkdo = function( callback, params, pkey, pvalue ) {
    var items = this.items;
    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        item.walkdo( callback, params, i, this );
    }

    callback( this, params, pkey, pvalue );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.items.is_inline = function() {
    var items = this.items;

    var l = items.length;
    return ( l === 0 ) || ( l === 1 && items[ 0 ].is_inline() );
};

asts.items.is_const = function() {
    var items = this.items;

    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        if ( !item.is_const() ) {
            return false;
        }
    }

    return true;
};

//  ---------------------------------------------------------------------------------------------------------------  //

//  ---------------------------------------------------------------------------------------------------------------  //

//  Q.  Чем items отличаются от collection?
//
//  A.  Тем, что items -- это выражение, например, все выражения в блоке,
//      а collection это просто набор чего-либо, например, elses в if
//      (содержит энное количество else_if и, возможно, else).
//

asts.collection = {};

asts.collection.options = {
    base: 'items'
};

asts.collection.get_type = function() {
    return TYPE.NONE;
};

asts.collection.is_inline = function() {
    var items = this.items;

    switch ( items.length ) {
        case 0: return true;
        case 1: return items[ 0 ].is_inline();
    }

    return false;
};


//  ## Module and block
//  ---------------------------------------------------------------------------------------------------------------  //

asts.module = {};

asts.module.options = {
    //  NOTE: Тут нет `contents`, хотя они и есть в ast.
    props: 'vars funcs templates imports'
};

asts.module._init = function() {
    this.vars = this.ast( 'module_vars' );
    this.funcs = this.ast( 'module_funcs' );
    this.templates = this.ast( 'module_templates' );
    this.imports = this.ast( 'module_imports' );
};

asts.module.w_def = function() {
    var funcs = this.funcs;

    //  FIXME: Неплохо бы это делать один раз глобально,
    //  а не на каждый модуль.
    for ( var i = 0, l = internal_funcs.length; i < l; i++ ) {
        var func = this.child( 'def_internal_func', internal_funcs[ i ] );
        funcs.add( func );
    }
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.module_vars = {};

asts.module_vars.options = {
    mixin: 'collection'
};

asts.module_vars.teya__sep = '\n';

asts.module_vars.js__sep__init = '\n';
asts.module_vars.js__sep__export = '\n';
asts.module_vars.js__sep__list = ', ';
asts.module_vars.js__sep__def = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.module_funcs = {};

asts.module_funcs.options = {
    mixin: 'collection'
};

asts.module_funcs.js__sep__define = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.module_templates = {};

asts.module_templates.options = {
    mixin: 'collection'
};

asts.module_templates.teya__sep = '\n';
asts.module_templates.js__sep = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.module_imports = {};

asts.module_imports.options = {
    mixin: 'collection'
};

asts.module_imports.teya__sep = '\n';
asts.module_imports.js__sep = '\n';
asts.module_imports.js__sep__init = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.block = {};

asts.block.options = {
    props: 'params defs exprs',
    scope: true
};

asts.block._init = function() {
    this.params = this.ast( 'template_params' );
    this.defs = this.ast( 'block_defs' );
    this.exprs = this.ast( 'block_exprs' );
};

asts.block._get_type = function() {
    return this.exprs.get_type();
};

asts.block.oncast = function( to ) {
    this.exprs.cast( to );
};

asts.block.is_inline = function() {
    return this.params.is_empty() && this.defs.is_empty() && this.exprs.is_inline();
};

asts.block.starts_with_attr = function() {
    return this.exprs.starts_with_attr();
};

asts.block.closes_attrs = function() {
    return this.exprs.closes_attrs();
};

asts.block.close_attrs = function( is_open, should_close ) {
    this.exprs.close_attrs( is_open, should_close );
};

/*
asts.block.w_prepare = function() {
    if ( this.get_type() !== TYPE.XML && this.to_type !== TYPE.XML ) {
        return;
    }

    var items = this.exprs.items;
    var _items = [];

    var is_opened = ( this.parent.id === 'xml' );
    for ( var i = 0, l = items.length; i < l; i++ ) {
        var item = items[ i ];

        if ( item.get_type() !== TYPE.ATTR ) {
            if ( is_opened ) {
                _items.push( this.exprs.child( 'close_attrs' ) );
                is_opened = false;
            }
        } else {
            is_opened = true;
        }

        _items.push( item );
    }

    //  FIXME: Можно ли это сделать как-то более элегантно? Без `this.parent`?
    if ( is_opened && this.parent.id === 'xml' ) {
        _items.push( this.exprs.child( 'close_attrs' ) );
    }

    this.exprs.items = _items;
};
*/

//  ---------------------------------------------------------------------------------------------------------------  //

asts.block_defs = {};

asts.block_defs.options = {
    mixin: 'collection'
};

asts.block_defs.teya__sep = '\n';
asts.block_defs.js__sep__def = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.block_exprs = {};

asts.block_exprs.options = {
    mixin: 'items'
};

asts.block_exprs.teya__sep = '\n\n';
asts.block_exprs.js__sep__output = '\n';

//  ---------------------------------------------------------------------------------------------------------------  //

/*
asts.block_params = {};

asts.block_params.options = {
    mixin: 'collection'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.block_param = {};

asts.block_param.options = {
    props: 'name value'
};

asts.block_param._get_type = function() {
    return this.value.get_type();
};

asts.block_param.is_inline = function() {
    return this.value.is_inline();
};

asts.block_param.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};
*/

//  ## Declarations
//  ---------------------------------------------------------------------------------------------------------------  //

asts.def = {};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_template = {};

asts.def_template.options = {
    base: 'def',
    props: 'type name args body',
    scope: true
};

asts.def_template._get_type = function() {
    return this.type || this.body.get_type();
};

asts.def_template.w_def = function() {
    this.scope.root.add_template( this );

    this.tid = this.state.tid++;
};

asts.def_template.w_validate = function() {
    var type = this.get_type();

    if ( type === TYPE.ANY || type === TYPE.UNDEF ) {
        this.error( 'INVALID_RETURN_TYPE', { type: type } );
    }
};

asts.def_template.w_prepare = function() {
    this.body.cast( this.type );

    //  FIXME: А на что тут влияет true или false?
    this.body.close_attrs( true );
};

asts.def_template.starts_with_attr = function() {
    return this.body.starts_with_attr();
};

asts.def_template.normalize_name = function() {
    return this.name.replace( /[-\.]/g, '_' );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_imported_template = {};

asts.def_imported_template.options = {
    props: 'name shortname'
};

asts.def_imported_template._init = function( params ) {
    this.namespace = params.namespace;
    this.def = params.def;
    this.import = params.import;

    this.shortname = this.def.name;
    this.name = ( this.namespace ) ? this.namespace + '.' + this.shortname : this.shortname;
};

asts.def_imported_template._get_type = function() {
    return this.def.get_type();
};

asts.def_imported_template.starts_with_attr = function() {
    return this.def.starts_with_attr();
};

asts.def_imported_template.normalize_name = function() {
    return this.name.replace( /[-\.]/g, '_' );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_var = {};

asts.def_var.options = {
    base: 'def',
    props: 'name value'
};

asts.def_var.w_def = function() {
    var vars = this.scope.vars;
    var name = this.name;

    if ( vars[ name ] ) {
        this.error( 'VAR_REDEFINITION', { name: name } );
    }

    this.vid = this.state.vid++;
    this.is_user = true;

    vars[ name ] = this;
};

asts.def_var._get_type = function() {
    return this.value.get_type();
};

asts.def_var.w_prepare = function() {
    if ( !this.value.is_inline() ) {
        this.value.rid++;
        this.value.aid++;
    }
    this.value.cast();
};

asts.def_var.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};

asts.def_var.is_global = function() {
    return this.scope.is_global();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.import = {};

asts.import.options = {
    props: 'filename namespace externals'
};

asts.import.w_def = function() {
    var filename = this.resolved_filename;

    if ( /\.teya$/.test( filename ) ) {
        var a_module = teya.parse( filename );

        var scope = this.scope.root;
        var namespace = this.namespace;

        var templates = a_module.templates.items;
        for ( var i = 0, l = templates.length; i < l; i++ ) {
            var template = templates[ i ];

            var proxy = template.child( 'def_imported_template', {
                namespace: namespace,
                def: template,
                import: this
            } );

            //  Сперва добавляем шаблон в scope. Если он добавился (т.е. это
            //  не повторное добавление), тогда добавляем в список шаблонов.
            //
            var added = scope.add_template( proxy );
            if ( added ) {
                this.root.templates.add( proxy );
            }
        }
    }
};

asts.import.w_action = function() {
    this.iid = this.state.iid++;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.externals = {};

asts.externals.options = {
    base: 'collection'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_func = {};

asts.def_func.options = {
    base: 'def',
    props: 'name args type body',
    scope: true
};

asts.def_func.w_def = function() {
    //  FIXME: Можно ли тут написать просто `this.scope.add_def_func( this )`?
    this.scope.root.add_def_func( this );

    this.fid = this.state.fid++;
    this.is_user = true;
};

asts.def_func.w_validate = function() {
    var type = this.get_type();

    if ( type === TYPE.ANY || type === TYPE.UNDEF ) {
        this.error( 'INVALID_RETURN_TYPE', { type: type } );
    }
};

asts.def_func._get_type = function() {
    if ( this.type ) {
        return this.type;
    }

    if ( this.__calc_type ) {
        this.error( 'UNTYPED_RECURSION' );
    }
    this.__calc_type = true;

    return this.body.get_type();
};

asts.def_func.w_prepare = function() {
    if ( this.body ) {
        this.body.cast();
    }
};

asts.def_func.get_full_name = function() {
    var full_name = this.__full_name;

    if ( !full_name ) {
        full_name = this.__full_name = this.name + '(' + this.get_signature() + ')';
    }

    return full_name;
};

asts.def_func.get_signature = function() {
    var signature = this.__signature;

    if ( !signature ) {
        signature = this.__signature = this.args.get_signature();
    }

    return signature;
};

asts.def_func.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};

asts.def_func.is_compatible_signature = function( inline_func_args ) {
    inline_func_args = inline_func_args.items;
    var args = this.args.items;

    var l1 = inline_func_args.length;
    var l2 = args.length;

    var last_arg = args[ l2 - 1 ];
    var rest_type;
    if ( last_arg ) {
       rest_type = types.is_rest( last_arg.get_type() );
    }

    if ( l1 > l2 && !rest_type ) {
        //  Пытаемся передать параметров больше, чем определено.
        return false;
    }

    var i = 0;
    while ( i < l1 ) {
        var inline_func_arg = inline_func_args[ i ];

        var arg_type = '';
        if ( rest_type && i >= l2 - 1 ) {
            arg_type = rest_type;

        } else {
            var arg = args[ i ];
            if ( arg ) {
                arg_type = arg.get_type();
            }
        }

        if ( !( arg_type && types.is_castable( inline_func_arg.get_type(), arg_type ) ) ) {
            return false;
        }

        i++;
    }

    //  Если в определении функции заявлено больше аргументов, чем передали,
    //  то все лишние должны иметь дефолтное значение.
    //
    while ( i < l2 ) {
        var arg = args[ i ];
        if ( !arg.default ) {
            return false;
        }

        i++;
    }

    return true;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_internal_func = {};

asts.def_internal_func.options = {
    base: 'def_func',
    props: 'name args type'
};

asts.def_internal_func._init = function( params ) {
    this.name = params.name;
    this.type = params.type;

    var arg_types = params.args;
    var args = this.args = this.child( 'def_args' );
    for ( var i = 0, l = arg_types.length; i < l; i++ ) {
        var arg_type = arg_types[ i ];
        var arg = args.child( 'def_arg', {
            type: arg_type
        } );
        args.add( arg );
    }
};

asts.def_internal_func.w_def = function() {
    //  FIXME: Можно ли тут написать просто `this.scope.add_def_func( this )`?
    this.scope.root.add_def_func( this );

    this.is_internal = true;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_external_func = {};

asts.def_external_func.options = {
    base: 'def_internal_func'
};

asts.def_external_func.w_def = function() {
    //  FIXME: Можно ли тут написать просто `this.scope.add_def_func( this )`?
    this.scope.root.add_def_func( this );

    this.is_external = true;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_args = {};

asts.def_args.options = {
    mixin: 'collection'
};

asts.def_args.teya__sep = ', ';
asts.def_args.js__sep__default = '\n';

asts.def_args.w_validate = function() {
    //  Validation.
    //  Проверяем, что после параметра с дефолтным значением,
    //  все остальные параметры тоже с дефолтным значением.

    var args = this.items;
    var has_default;
    for ( var i = 0, l = args.length; i < l; i++ ) {
        var arg = args[ i ];
        if ( has_default && !arg.default ) {
            arg.error( 'DEFAULT_VALUE_REQUIRED' );
        }
        if ( arg.default ) {
            has_default = true;
        }
    }
};


asts.def_args.index_of = function( name ) {
    var args = this.items;
    for ( var i = 0, l = args.length; i < l; i++ ) {
        var arg = args[ i ];

        if ( arg.name === name ) {
            return i;
        }
    }

    return -1;
};

asts.def_args.get_signature = function() {
    var signature = '';

    var args = this.items;
    for ( var i = 0, l = args.length; i < l; i++ ) {
        var arg = args[ i ];

        if ( i ) {
            signature += ',';
        }
        signature += arg.get_signature();
    }

    return signature;
};


//  ---------------------------------------------------------------------------------------------------------------  //

asts.def_arg = {};

asts.def_arg.options = {
    props: 'type name default'
};

asts.def_arg._init = function( params ) {
    if ( params ) {
        this.name = params.name;
        this.type = params.type;
        this.default = params.default;
    }
};

asts.def_arg.w_action = function() {
    if ( !this.type && !this.default ) {
        this.error( 'UNKNOWN_ARG_TYPE' );
    }

    var name = this.name;
    if ( name ) {
        var vars = this.scope.vars;

        if ( vars[ name ] ) {
            this.error( 'VAR_REDEFINITION', { name: name } );
        }

        this.vid = this.state.vid++;
        this.is_arg = true;

        vars[ name ] = this;
    }
};

asts.def_arg._get_type = function() {
    return this.type || this.default.get_type();
};

asts.def_arg.w_prepare = function() {
    if ( this.default ) {
        this.default.cast( this.get_type() );
    }
};

asts.def_arg.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};

asts.def_arg.get_signature = function() {
    return this.get_type();
};


//  ## Expressions
//  ---------------------------------------------------------------------------------------------------------------  //

asts.expr = {};


//  ### Block expressions
//  ---------------------------------------------------------------------------------------------------------------  //

asts.if = {};

asts.if.options = {
    base: 'expr',
    props: 'condition then elses'
};

asts.if._init = function() {
    this.elses = this.ast( 'elses' );
};

asts.if._get_type = function() {
    var type = this.then.get_type();

    var elses = this.elses.items;
    for ( var i = 0, l = elses.length; i < l; i++ ) {
        type = types.common_type( type, elses[ i ].get_type() );
    }

    return type;
};

asts.if.w_prepare = function() {
    this.condition.cast( TYPE.BOOLEAN );

    var elses = this.elses.items;
    for ( var i = 0, l = elses.length; i < l; i++ ) {
        var condition =  elses[ i ].condition;
        if ( condition ) {
            condition.cast( TYPE.BOOLEAN );
        }
    }
};

asts.if.oncast = function( to ) {
    this.then.cast( to );
    this.elses.cast( to );
};

asts.if.is_inline = function() {
    return this.then.is_inline() && this.elses.is_inline();
};

asts.if.starts_with_attr = function() {
    return this.then.starts_with_attr() || this.elses.starts_with_attr();
};

asts.if.closes_attrs = function() {
    return this.then.closes_attrs() && this.elses.closes_attrs();
};

asts.if.close_attrs = function( is_open ) {
    this.then.close_attrs( is_open );
    this.elses.close_attrs( is_open );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.elses = {};

asts.elses.options = {
    mixin: 'collection'
};

asts.elses.teya__sep = '\n';
asts.elses.js__sep__if_body = ' ';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.else_if = {};

asts.else_if.options = {
    props: 'condition body'
};

asts.else_if._get_type = function() {
    return this.body.get_type();
};

//  NOTE: is_inline === no.false
//  Поэтому, если есть хоть один else if, то if не может быть is_inline.

asts.else_if.oncast = function( to ) {
    this.body.cast( to );
};

asts.else_if.starts_with_attr = function() {
    return this.body.starts_with_attr();
};

asts.else_if.closes_attrs = function() {
    return this.body.closes_attrs();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.else = {};

asts.else.options = {
    props: 'body'
};

asts.else._get_type = function() {
    return this.body.get_type();
};

asts.else.is_inline = function() {
    return this.body.is_inline();
};

asts.else.oncast = function( to ) {
    this.body.cast( to );
};

asts.else.starts_with_attr = function() {
    return this.body.starts_with_attr();
};

asts.else.closes_attrs = function() {
    return this.body.closes_attrs();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.for = {};

asts.for.options = {
    base: 'expr',
    props: 'selector body'
};

asts.for._get_type = function() {
    var type = this.body.get_type();

    return types.join_types( type, type );
};

asts.for.w_prepare = function() {
    this.body.xid++;

    this.body.cast( this.get_type() );
};

asts.for.starts_with_attr = function() {
    return this.body.starts_with_attr();
};

asts.for.closes_attrs = function() {
    return this.body.closes_attrs();
};

asts.for.close_attrs = function( is_open ) {
    this.body.close_attrs( is_open );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.with = {};

asts.with.options = {
    base: 'expr',
    props: 'selector body'
};

asts.with._get_type = function() {
    return this.body.get_type();
};

asts.with.w_prepare = function() {
    this.body.cast( this.get_type() );
};

asts.with.is_inline = function() {
    return this.body.is_inline();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.template = {};

asts.template.options = {
    base: 'expr',
    props: 'name params content'
};

asts.template._get_type = function() {
    //  FIXME: Может ли результат выполнения шаблона
    //  быть любого типа? Или только, скажем, xml и attr?
    //
    return this.def.get_type();
    /// return TYPE.XML;
};

asts.template.w_action = function() {
    var name = this.name;

    var def = this.scope.find_template( name );
    //  NOTE: Кажется, не может быть так, чтобы def был неопределен.
    //  Иначе этот идентификатор не будет считаться template,
    //  а будет var.

    this.def = def;

    var inline_params = this.params;
    var block_params = this.content.params;
    var params = this.child( 'template_params' );
    var args = def.args;

    if ( args ) {
        var that = this;

        if ( inline_params ) {
            inline_params.for_each( function( param, i ) {
                var arg = args.at( i );

                if ( !arg ) {
                    param.error( 'TOO_MANY_PARAMS' );
                }

                params.at( i, that.child( 'template_param', {
                    name: arg.name,
                    value: param.value,
                    def: arg
                } ) );
            } );
        }

        block_params.for_each( function( param ) {
            name = param.name;
            var i = args.index_of( name );

            if ( i === -1 ) {
                param.error( 'NO_SUCH_ARGUMENT', { name: name } );
            }
            if ( params.at( i ) ) {
                param.error( 'PARAM_ALREADY_USED', { name: name } );
            }

            params.at( i, that.child( 'template_param', {
                name: name,
                value: param.value,
                def: args.at( i )
            } ) );
        } );

        args.for_each( function( arg, i ) {
            var param = params.at( i );

            if ( !param ) {
                params.at( i, that.child( 'template_param', {
                    name: arg.name,
                    value: null,
                    def: arg
                } ) );
            }
        } );
    }

    this.params = params;
    this.content.params = null;
};

asts.template.w_prepare = function() {
    this.content.rid++;

    this.content.cast( TYPE.XML );

    this.params.for_each( function( param ) {
        if ( param.value ) {
            param.cast( param.def.get_type() );
        }
    } );
};

asts.template.close_attrs = function( is_open ) {

};

asts.template.starts_with_attr = function() {
    return this.def.starts_with_attr();
};

asts.template.normalize_name = function() {
    return this.name.replace( /[-\.]/g, '_' );
};

//  ---------------------------------------------------------------------------------------------------------------  //

/*
asts.inline_params = {};

asts.inline_params.options = {
    mixin: 'collection'
};

asts.inline_params.teya__sep = ', ';

asts.inline_param = {};

asts.inline_param.options = {
    props: 'name value'
};

asts.inline_param._get_type = function() {
    return this.value.get_type();
};

asts.inline_param.is_inline = no.true;

asts.inline_param.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};
*/

//  ---------------------------------------------------------------------------------------------------------------  //

asts.template_params = {};

asts.template_params.options = {
    mixin: 'collection'
};

asts.template_params.teya__sep = ', ';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.template_param = {};

asts.template_param.options = {
    props: 'name value'
};

asts.template_param._init = function( params ) {
    if ( params ) {
        this.name = params.name;
        this.value = params.value;
        //  FIXME: Тут нельзя просто так брать и копировать ссылки. Нужно делать append,
        //  с заменой parent'а и всего остального. Нужен специальный метод.
        //  this.value.parent = this;
        this.def = params.def;
    }
};

asts.template_param._get_type = function() {
    var value = this.value;

    if ( value ) {
        return this.value.get_type();
    }

    return TYPE.NONE;
};

asts.template_param.is_inline = function() {
    if ( !this.value ) {
        return true;
    }

    return this.value.is_inline();
};

asts.template_param.normalize_name = function() {
    return this.name.replace( /-/g, '_' );
};

asts.template_param.w_prepare = function() {
    if ( this.value && !this.value.is_inline() ) {
        this.value.rid++;
        this.value.aid++;
    }
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.value = {};

asts.value.options = {
    base: 'expr',
    props: 'value'
};

asts.value._init = function( params ) {
    this.value = params;
};

asts.value._get_type = function() {
    return this.value.get_type();
};

asts.value.is_inline = function() {
    return this.value.is_inline();
};

asts.value.oncast = function( to ) {
    this.value.cast( to );
};

//  FIXME: Отдельно нужно учитывать inline_expr.
asts.value.starts_with_attr = function() {
    return ( this.value.get_type() === TYPE.ATTR );
};

//  FIXME: Отдельно нужно учитывать inline_expr.
asts.value.closes_attrs = function() {
    return ( this.value.get_type() !== TYPE.ATTR );
};

asts.value.close_attrs = no.op;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.subexpr = {};

asts.subexpr.options = {
    base: 'expr',
    props: 'body'
};

asts.subexpr._get_type = function() {
    return this.body.get_type();
};

asts.subexpr.is_inline = function() {
    this.body.is_inline();
};

asts.subexpr.oncast = function( to ) {
    this.body.cast( to );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.array = {};

asts.array.options = {
    base: 'expr',
    props: 'body'
};

asts.array._get_type = function() {
    return TYPE.ARRAY;
};

asts.array.is_inline = function() {
    return this.body.is_inline();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.object = {};

asts.object.options = {
    base: 'expr',
    props: 'body'
};

asts.object._get_type = function() {
    return TYPE.OBJECT;
};

asts.object.is_inline = function() {
    return this.body.is_inline();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.pair = {};

asts.pair.options = {
    base: 'expr',
    props: 'key value'
};

asts.pair._init = function( params ) {
    this.key = params.key;
    this.value = params.value;
};

asts.pair._get_type = function() {
    return TYPE.PAIR;
};

asts.pair.w_prepare = function() {
    this.value.rid++;

    this.key.cast( TYPE.SCALAR );
    this.value.cast();
};

asts.pair.is_inline = function() {
    return this.value.is_inline();
};


//  #### XML
//  ---------------------------------------------------------------------------------------------------------------  //

asts.xml = {};

asts.xml.options = {
    base: 'expr',
    props: 'name xml_classes xml_id attrs content'
};

asts.xml._get_type = function() {
    return TYPE.XML;
};

asts.xml.w_prepare = function() {
    this.content.cast( TYPE.XML );

    if ( !this.name.is_const() ) {
        this.nid++;
        this.name.cast( TYPE.STRING );
    } else {
        this.is_empty_tag = R.is_empty_tag( this.name.as_string() );
    }

    this.content.close_attrs(
        this.content.starts_with_attr(),
        //  Флаг, говорящий о том, что в этом блоке нужен всегда `close_attrs`.
        true
    );
};

asts.xml.starts_with_attr = no.false;

asts.xml.closes_attrs = no.true;

asts.xml.close_attrs = no.op;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.xml_attrs = {};

asts.xml_attrs.options = {
    mixin: 'collection'
};

asts.xml_attrs.js__sep__inline = ' + ';
asts.xml_attrs.js__sep__output = ',\n';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.xml_attr = {};

asts.xml_attr.options = {
    base: 'expr',
    props: 'name value'
};

asts.xml_attr._get_type = function() {
    return TYPE.ATTR;
};

asts.xml_attr.w_prepare = function() {
    //  FIXME: Тут должно быть attrvalue?
    this.value.cast( TYPE.STRING );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.xml_classes = {};

asts.xml_classes.options = {
    mixin: 'collection'
};

asts.xml_class = {};

asts.xml_class.options = {
    props: 'value'
};

asts.xml_id = {};

asts.xml_id.options = {
    props: 'value'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.attr = {};

asts.attr.options = {
    base: 'expr',
    props: 'name value'
};

asts.attr._get_type = function() {
    return TYPE.ATTR;
};

asts.attr.w_prepare = function() {
    var type = this.value.get_type();
    if ( type === TYPE.OBJECT || type === TYPE.ARRAY ) {
        this.value.cast( TYPE.JSON );

    } else {
        this.value.cast( TYPE.STRING );
    }

    if ( !this.value.is_inline() ) {
        this.value.rid++;
    }
};

asts.attr.starts_with_attr = no.true;

asts.attr.closes_attrs = no.false;

asts.attr.close_attrs = no.op;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.param_content = {};

asts.param_content.options = {
    base: 'expr'
};

asts.param_content._get_type = function() {
    return TYPE.XML;
};

asts.param_content.starts_with_attr = no.op;

asts.param_content.close_attrs = no.op;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.param_content_attrs = {};

asts.param_content_attrs.options = {
    base: 'expr'
};

asts.param_content_attrs._get_type = function() {
    return TYPE.ATTR;
};

asts.param_content_attrs.starts_with_attr = no.true;

asts.param_content_attrs.close_attrs = no.op;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.param_content_other = {};

asts.param_content_other.options = {
    base: 'expr'
};

asts.param_content_other._get_type = function() {
    return TYPE.XML;
};

asts.param_content_other.starts_with_attr = no.false;

asts.param_content_other.close_attrs = no.op;


//  ### Inline expressions
//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_expr = {};

asts.inline_expr.is_inline = no.true;

//  #### Binary operators
//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_binop = {};

asts.inline_binop.options = {
    base: 'inline_expr',
    props: 'left op right'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_or = {};

asts.inline_or.options = {
    base: 'inline_binop'
};

asts.inline_or._get_type = function() {
    //  Почему тут берется тип правого аргумента?
    //  Потому что, если левый аргумент ложен, то мы получим правый аргумент.
    //  Если же он истинен, то как минимум должен приводиться к типу правого аргумента.
    //
    return this.right.get_type();
};

asts.inline_or.w_prepare = function() {
    var type = this.get_type();

    this.left.cast( type );
    this.right.cast( type );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_and = {};

asts.inline_and.options = {
    base: 'inline_binop'
};

asts.inline_and._get_type = function() {
    //  См. комментарии к inline_or._get_type.
    //
    return this.right.get_type();
};

asts.inline_and.w_prepare = function() {
    var type = this.get_type();

    this.left.cast( type );
    this.right.cast( type );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_eq = {};

asts.inline_eq.options = {
    base: 'inline_binop'
};

asts.inline_eq._get_type = function() {
    return TYPE.BOOLEAN;
};

asts.inline_eq.w_prepare = function() {
    var left = this.left;
    var right = this.right;

    var left_type = left.get_type();
    var right_type = right.get_type();

    if ( left === TYPE.BOOLEAN || right === TYPE.BOOLEAN ) {
        left.cast( TYPE.BOOLEAN );
        right.cast( TYPE.BOOLEAN );
    } else {
        left.cast( TYPE.SCALAR );
        right.cast( TYPE.SCALAR );
    }
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_rel = {};

asts.inline_rel.options = {
    base: 'inline_binop'
};

asts.inline_rel._get_type = function() {
    return TYPE.BOOLEAN;
};

asts.inline_rel.w_prepare = function() {
    this.left.cast( TYPE.SCALAR );
    this.right.cast( TYPE.SCALAR );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_add = {};

asts.inline_add.options = {
    base: 'inline_binop'
};

asts.inline_add._get_type = function() {
    return TYPE.NUMBER;
};

asts.inline_add.w_prepare = function() {
    this.left.cast( TYPE.NUMBER );
    this.right.cast( TYPE.NUMBER );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_mul = {};

asts.inline_mul.options = {
    base: 'inline_binop'
};

asts.inline_mul._get_type = function() {
    return TYPE.NUMBER;
};

asts.inline_mul.w_prepare = function() {
    this.left.cast( TYPE.NUMBER );
    this.right.cast( TYPE.NUMBER );
};

//  ---------------------------------------------------------------------------------------------------------------  //

//  FIXME: Подумать еще, а нужен ли этот оператор?
//
asts.inline_union = {};

asts.inline_union.options = {
    base: 'inline_binop'
};

asts.inline_union._get_type = function() {
    return TYPE.ARRAY;
};

asts.inline_union.w_prepare = function() {
    this.left.cast( TYPE.ARRAY );
    this.right.cast( TYPE.ARRAY );
};

//  #### Unary operators
//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_unop = {};

asts.inline_unop.options = {
    base: 'inline_expr',
    props: 'left op'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_unary = {};

asts.inline_unary.options = {
    base: 'inline_unop'
};

asts.inline_unary._get_type = function() {
    return TYPE.NUMBER;
};

asts.inline_unary.w_prepare = function() {
    this.left.cast( TYPE.NUMBER );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_not = {};

asts.inline_not.options = {
    base: 'inline_unop'
};

asts.inline_not._get_type = function() {
    return TYPE.BOOLEAN;
};

asts.inline_not.w_prepare = function() {
    this.left.cast( TYPE.BOOLEAN );
};

//  #### Inline primaries
//  ---------------------------------------------------------------------------------------------------------------  //

asts.true = {};

asts.true.options = {
    base: 'inline_expr'
};

asts.true._get_type = function() {
    return TYPE.BOOLEAN;
};

asts.false = {};

asts.false.options = {
    base: 'inline_expr'
};

asts.false._get_type = function() {
    return TYPE.BOOLEAN;
};


//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_number = {};

asts.inline_number.options = {
    base: 'inline_expr',
    props: 'value'
};

asts.inline_number._get_type = function() {
    return TYPE.NUMBER;
};

asts.inline_number.is_const = no.true;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_subexpr = {};

asts.inline_subexpr.options = {
    base: 'inline_expr',
    props: 'expr'
};

asts.inline_subexpr._get_type = function() {
    return this.expr.get_type();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_var = {};

asts.inline_var.options = {
    base: 'inline_expr',
    props: 'name'
};

asts.inline_var.w_action = function() {
    var name = this.name;

    var def = this.scope.find_var( name );
    if ( !def ) {
        this.error( 'UNDEFINED_VAR', { name: name } );
    }

    this.def = def;
};

asts.inline_var._get_type = function() {
    return this.def.get_type();
};

asts.inline_var.is_global = function() {
    return this.def.is_global();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_func = {};

asts.inline_func.options = {
    base: 'inline_expr',
    props: 'name args'
};

asts.inline_func._get_type = function() {
    return this.def.get_type();
};

asts.inline_func.w_action = function() {
    //  var name = this.name;

    var def = this.scope.find_def_func( this );
    //  if ( !def ) {
    //      this.error( 'UNDEFINED_FUNC', { name: name } );
    //  }

    this.def = def;
};

asts.inline_func.w_prepare = function() {
    var def_args = this.def.args;
    var args = this.args;

    var rest_type;
    this.args.for_each( function( arg, i ) {
        var def_arg = def_args.at( i );
        if ( !def_arg && !rest_type ) {
            arg.error( 'ARG_NOT_DEFINED' );
        }

        var type;
        if ( !rest_type ) {
            type = def_arg.get_type();
            rest_type = types.is_rest( type );
        }

        arg.value.cast( rest_type || type );
    } );
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_func_args = {};

asts.inline_func_args.options = {
    mixin: 'collection'
};

asts.inline_func_args.teya__sep = ', ';
asts.inline_func_args.js__sep = ', ';

asts.inline_func_arg = {};

asts.inline_func_arg.options = {
    props: 'name value'
};

asts.inline_func_arg._get_type = function() {
    return this.value.get_type();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_ternary = {};

asts.inline_ternary.options = {
    base: 'inline_expr',
    props: 'condition then else'
};

asts.inline_ternary._get_type = function() {
    return types.common_type( this.then.get_type(), this.else.get_type() );
};


//  ##### Inline stirng
//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_string = {};

asts.inline_string.options = {
    base: 'inline_expr',
    props: 'value'
};

asts.inline_string._get_type = function() {
    return TYPE.STRING;
};

asts.inline_string.oncast = function( to ) {
    this.value.cast( to );
};

asts.inline_string.as_string = function() {
    return this.value.as_string();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.string_content = {};

asts.string_content.options = {
    mixin: 'collection'
};

asts.string_content.teya__sep = '';
asts.string_content.js__sep__cast = ' + ';

asts.string_content.as_string = function() {
    var items = this.items;

    //  FIXME: Работает только, если там константная строка.
    return items[ 0 ].value;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.string_expr = {};

asts.string_expr.options = {
    base: 'inline_expr',
    props: 'expr'
};

asts.string_expr._get_type = function() {
    return this.expr.get_type();
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.string_literal = {};

asts.string_literal.options = {
    base: 'inline_expr',
    props: 'value'
};

asts.string_literal._get_type = function() {
    return TYPE.STRING;
};

asts.string_literal.w_action = function() {
    this.value = deentitify( this.value );
};

asts.string_literal.stringify = function() {
    var to_type = this.to_type;
    //  FIXME: Сделать какой-то types.need_cast.
    var value = ( to_type && to_type !== TYPE.ANY ) ? R[ 'to_' + to_type ]( this.value ) : this.value;

    return value
        .replace( /"/g, '\\"', 'g' )
        .replace( /'/g, "\\'", 'g' )
        .replace( /\n/g, '\\n', 'g' );
};

asts.string_literal.is_const = no.true;

//  #### JPath
//  ---------------------------------------------------------------------------------------------------------------  //

asts.jpath = {};

asts.jpath.options = {
    base: 'inline_expr',
    props: 'abs steps in_context'
};

asts.jpath._get_type = function() {
    return TYPE.JSON;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.jpath_steps = {};

asts.jpath_steps.options = {
    mixin: 'collection'
};

asts.jpath_steps.teya__sep = '';

//  ---------------------------------------------------------------------------------------------------------------  //

asts.jpath_nametest = {};

asts.jpath_nametest.options = {
    props: 'name'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.jpath_predicate = {};

asts.jpath_predicate.options = {
    props: 'expr'
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.jpath_filter = {};

asts.jpath_filter.options = {
    base: 'inline_expr',
    props: 'jpath expr'
};

asts.jpath_filter._get_type = function() {
    return TYPE.JSON;
};

asts.jpath_filter.w_prepare = function() {
    this.expr.cast( TYPE.JSON );
};


//  #### Inline misc
//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_object = {};

asts.inline_object.options = {
    base: 'inline_expr',
    props: 'body'
};

asts.inline_object._get_type = function() {
    return TYPE.OBJECT;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_pairs = {};

asts.inline_pairs.options = {
    mixin: 'items'
};

asts.inline_pairs.teya__sep = ', ';
asts.inline_pairs.js__sep__output = '\n';
asts.inline_pairs.js__sep__value = ', ';

asts.inline_pairs.is_inline = no.true;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_pair = {};

asts.inline_pair.options = {
    props: 'key value'
};

asts.inline_pair._init = function( params ) {
    if ( params ) {
        this.key = params.key;
        this.value = params.value;
    }
};

asts.inline_pair._get_type = function() {
    return TYPE.PAIR;
};

asts.inline_pair.w_prepare = function() {
    this.key.cast( TYPE.SCALAR );
    this.value.cast();
};

asts.inline_pair.is_inline = no.true;

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_array = {};

asts.inline_array.options = {
    base: 'inline_expr',
    props: 'body'
};

asts.inline_array._get_type = function() {
    return TYPE.ARRAY;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_items = {};

asts.inline_items.options = {
    mixin: 'items'
};

asts.inline_items.teya__sep = ', ';
asts.inline_items.js__sep__output = '\n';
asts.inline_items.js__sep__value = ', ';

asts.inline_items.is_inline = function() {
    return true;
};

//  ---------------------------------------------------------------------------------------------------------------  //

asts.inline_item = {};

asts.inline_item.options = {
    props: 'value'
};

asts.inline_item._init = function( params ) {
    this.value = params;
};

asts.inline_item._get_type = function() {
    return TYPE.ITEM;
};

asts.inline_item.is_inline = no.true;


//  ---------------------------------------------------------------------------------------------------------------  //

asts.close_attrs = {};

asts.close_attrs.options = {
};

asts.close_attrs._get_type = function() {
    return TYPE.XML;
};

asts.close_attrs.starts_with_attr = no.false;

asts.close_attrs.closes_attrs = no.true;

asts.close_attrs.close_attrs = no.op;


//  ---------------------------------------------------------------------------------------------------------------  //

//  Добавляем кодогенерацию.

require( '../templates/compiled/teya.js' )( asts );
require( '../templates/compiled/js.js' )( asts );

//  ---------------------------------------------------------------------------------------------------------------  //

module.exports = asts;

//  ---------------------------------------------------------------------------------------------------------------  //

