module.exports = function( params ) {
    var Plugin = params.Plugin;
    var t = params.types;

    return new Plugin( 'test', {
        visitor: {
            Program: function( node ) {
                //  Добавляем в начало файла:
                //
                //      var indent = require( './util.js' ).indent;
                //
                node.body.unshift(
                    t.variableDeclaration(
                        'var',
                        [
                            t.variableDeclarator(
                                t.identifier( 'indent' ),
                                t.memberExpression(
                                    call( 'require', [ t.literal( '../lib/runtime.js' ) ] ),
                                    t.identifier( 'indent' )
                                )
                            ),

                            t.variableDeclarator(
                                t.identifier( 'line' ),
                                t.memberExpression(
                                    call( 'require', [ t.literal( '../lib/runtime.js' ) ] ),
                                    t.identifier( 'line' )
                                )
                            )
                        ]
                )
                );
            },

            TemplateLiteral: function( node ) {
                var literals = node.quasis;
                var exprs = node.expressions;
                //  В literals всегда на один элемент больше, чем в exprs.
                var l = exprs.length;

                var groups = [];
                var group = null;

                //  Если мы пишем template literal в виде:
                //
                //      return `
                //          foobar
                //      `
                //
                //  то нужно в начале отрезать перевод строки.
                //  А в конце переводы строк и/или пробелы.
                //
                var first = literals[ 0 ].value.cooked;
                var indent;
                if ( first.charAt( 0 ) === '\n' ) {
                    first = first.substr( 1 );
                    //  Запоминаем, глобальный индент.
                    //  Потом нужно будет вырезать его из всех строк.
                    indent = leading_spaces( first ).length;

                    literals[ l ].value.cooked = literals[ l ].value.cooked.trimRight();
                }

                //  Группируем литералы и выражения по строкам.
                //  Каждый перевод строки внутри литерала заканчивает группу/строку и начинает новую.
                push_literal( first );
                for ( var i = 0, l = exprs.length; i < l; i++ ) {
                    group.push( exprs[ i ] );
                    push_literal( literals[ i + 1 ].value.cooked );
                }

                //  log( groups );

                //  Генерируем AST, складывающее все литералы и выражения.
                for ( var i = 0, l = groups.length; i < l; i++ ) {
                    var group = groups[ i ];

                    var is_not_last = ( i < l - 1 );

                    var first = group[ 0 ];

                    if ( group.length === 1 ) {
                        //  Если группа состоит из одного элемента, то это точно литерал.
                        //
                        if ( is_not_last ) {
                            first += '\n';
                        }
                        groups[ i ] = t.literal( first );

                    } else {
                        //  Первый элемент группы всегда литерал.
                        //
                        //  Смотрим, нет ли еще индента. Если есть, запоминаем и отрезаем.
                        var line_indent = leading_spaces( first );
                        if ( line_indent ) {
                            first = first.substr( line_indent.length );
                        }
                        group[ 0 ] = first;

                        var is_not_empty = is_not_empty_line( group );

                        var r;
                        if ( is_not_empty ) {
                            if ( is_not_last ) {
                                group.push( '\n' );
                            }
                            r = reduce_group( group );
                            if ( line_indent ) {
                                r = call( 'indent', [ r, t.literal( line_indent ) ] )
                            }

                        } else {
                            r = reduce_group( group );
                            if ( is_not_last ) {
                                r = call( 'line', [ r ] );
                            }
                            if ( line_indent ) {
                                r = call( 'indent', [ r, t.literal( line_indent ) ] );
                            }
                        }
                        groups[ i ] = r;
                    }
                }
                //  "Суммируем" все группы в одно выражение.
                var r = reduce_group( groups );

                return r;

                function push_literal( literal ) {
                    var lines = literal.split( '\n' );
                    var l = lines.length;

                    if ( l === 1 ) {
                        push_string( lines[ 0 ] );

                    } else {
                        //  Каждая строка, кроме последней, заканчивает группу и начинает новую.
                        //  В случае, если строка всего одна (она же будет и последней), то группа не меняется.
                        //
                        for ( var i = 0, l = lines.length; i < l - 1; i++ ) {
                            push_string( lines[ i ] );
                            group = null;
                        }
                        push_string( lines[ l - 1 ] );
                    }
                }

                function push_string( str ) {
                    if ( !group ) {
                        str = deindent_string( str, indent );

                        group = [];
                        groups.push( group );
                    }

                    group.push( str );
                }

            }
        }
    } );

    function is_not_empty_line( line ) {
        for ( var i = 0, l = line.length; i < l; i++ ) {
            var item = line[ i ];
            if ( item && typeof item === 'string' ) {
                return true;
            }
        }
    }

    function call( name, args ) {
        return t.callExpression(
            t.identifier( name ),
            args
        );
    }

    function reduce_group( group ) {
        //  Берем первый элемент и "прибавляем" к нему все остальные.
        var r = wrap_literal( group[ 0 ] );
        for ( var i = 1, l = group.length; i < l; i++ ) {
            r = t.binaryExpression( '+', r, wrap_literal( group[ i ] ) );
        }
        return r;
    }

    //  Если `s` это строка, то делаем из нее литерал,
    //  иначе возвращаем как есть.
    //
    function wrap_literal( s ) {
        return ( typeof s === 'string' ) ? t.literal( s ) : s;
    }

}

var rx_spaces = /^(\ +)/;
function leading_spaces( s ) {
    var r = rx_spaces.exec( s );
    if ( r ) {
        return r[ 1 ];
    }

    return '';
}

function deindent_string( s, n ) {
    var l = leading_spaces( s ).length;
    if ( !l ) {
        return s;
    }

    if ( l < n ) {
        return s.substr( l );
    }

    return s.substr( n );
}

//  ---------------------------------------------------------------------------------------------------------------  //

function log( o ) {
    console.log( JSON.stringify(
        o,
        function( key, value ) {
            if ( key === '_scopeInfo' || key === '_paths' ) {
                return;
            }

            return value;
        },
        4
    ) );
}

