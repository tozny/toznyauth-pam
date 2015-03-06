#[macro_export]
macro_rules! mdo {
    (
        let $p: pat = $e: expr ; $( $t: tt )*
    ) => (
        { let $p = $e ; mdo! { $( $t )* } }
    );

    (
        let $p: ident : $ty: ty = $e: expr ; $( $t: tt )*
    ) => (
        { let $p: $ty = $e ; mdo! { $( $t )* } }
    );

    (
        $p: pat =<< $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |$p| mdo! { $( $t )* } )
    );

    (
        $p: ident : $ty: ty =<< $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |$p : $ty| mdo! { $( $t )* } )
    );

    (
        ign $e: expr ; $( $t: tt )*
    ) => (
        bind($e, |_| mdo! { $( $t )* })
    );

    (
        when $e: expr ; $( $t: tt )*
    ) => (
        bind(if $e { ret(()) } else { mzero() }, |_| mdo! { $( $t )* })
    );

    (
        ret $f: expr
    ) => (
        $f
    )
}
