use gridcraft_formula::{FormulaLocale, parse, parse_input, print, print_input};

fn input(src: &str) -> String {
    print(&parse_input(src, FormulaLocale::Fr, |_| false).unwrap())
}

fn display(canonical: &str) -> String {
    print_input(&parse(canonical).unwrap(), FormulaLocale::Fr, |_| false).unwrap()
}

#[test]
fn french_names_and_separators() {
    for (src, expected) in [
        ("=somme(A1:A10)", "SUM(A1:A10)"),
        ("SI(A1>0;\"Oui\";\"Non\")", "IF(A1>0,\"Oui\",\"Non\")"),
        ("RECHERCHEV(A2;D:E;2;FAUX)", "VLOOKUP(A2,D:E,2,FALSE)"),
        ("MOYENNE.SI.ENS(A1:A9;B1:B9;\">0\")", "AVERAGEIFS(A1:A9,B1:B9,\">0\")"),
        ("NB.SI(A1:A9;\">0\")+NBVAL(A1:A9)", "COUNTIF(A1:A9,\">0\")+COUNTA(A1:A9)"),
        ("RECHERCHEX(A1;B:B;C:C)+AUJOURDHUI()", "XLOOKUP(A1,B:B,C:C)+TODAY()"),
        ("ECARTYPE.PEARSON(A1:A5)+VAR.P.N(A1:A5)", "STDEV.P(A1:A5)+VAR.P(A1:A5)"),
        ("SOMME(1,5;2,25)", "SUM(1.5,2.25)"),
        ("=vrai", "TRUE"),
        ("FAUX()", "FALSE()"),
        // Names French Excel keeps in English stay as typed.
        ("MAX(1;2)+INDEX(A1:B2;1;2)", "MAX(1,2)+INDEX(A1:B2,1,2)"),
        // French gives some English names to other functions, as French Excel does.
        ("TRIM(A1:A3;0,1;0,2)+VAR.P(A1:A3)", "MIRR(A1:A3,0.1,0.2)+VARP(A1:A3)"),
        // Spanish and German words mean nothing in French.
        ("SUMA(1;2)+WAHR", "SUMA(1,2)+WAHR"),
        ("LAMBDA(x;x+1)(2)", "LAMBDA(x,x+1)(2)"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
    assert!(parse("SOMME(1;2)").is_err());
}

#[test]
fn french_arrays_use_a_point_between_columns() {
    for (src, expected) in
        [("SOMME({1.2;3.4};5)", "SUM({1,2;3,4},5)"), ("{1,5.2,25}", "{1.5,2.25}"), ("{VRAI.FAUX;TRUE.FALSE}", "{TRUE,FALSE;TRUE,FALSE}")]
    {
        assert_eq!(input(src), expected, "{src}");
    }
    assert!(parse_input("{SOMME(A1)}", FormulaLocale::Fr, |_| false).is_err());
    // Spanish booleans are not French array constants.
    assert!(parse_input("{VERDADERO}", FormulaLocale::Fr, |_| false).is_err());
}

#[test]
fn french_display_of_canonical_formulas() {
    for (canonical, expected) in [
        ("IF(A1>0,SUM(1.5,2),FALSE)", "SI(A1>0;SOMME(1,5;2);FAUX)"),
        ("VLOOKUP(A2,D:E,2,TRUE)", "RECHERCHEV(A2;D:E;2;VRAI)"),
        ("{1.5,2;3,4}", "{1,5.2;3.4}"),
        ("MAX(1,2)", "MAX(1;2)"),
        ("TRIM(A1)+MIRR(A1:A3,0.1,0.2)", "SUPPRESPACE(A1)+TRIM(A1:A3;0,1;0,2)"),
    ] {
        assert_eq!(display(canonical), expected, "{canonical}");
        assert_eq!(input(&display(canonical)), canonical, "{canonical} round trip");
    }
}

#[test]
fn french_error_values() {
    for (src, expected) in [
        ("SI(ESTERREUR(A1);#VALEUR!;A1)", "IF(ISERROR(A1),#VALUE!,A1)"),
        ("SIERREUR(A1;#N/A)+#NOMBRE!", "IFERROR(A1,#N/A)+#NUM!"),
        ("{#NOM?.#NUL!;#DIV/0!.#REF!}", "{#NAME?,#NULL!;#DIV/0!,#REF!}"),
        ("#PROPAGATION!&#CALC!&#CHARGEMENT_DONNEES", "#SPILL!&#CALC!&#GETTING_DATA"),
        ("#nom?+#valeur!", "#NAME?+#VALUE!"),
        // The canonical spelling is still understood.
        ("#NULL!+#VALUE!+#NAME?", "#NULL!+#VALUE!+#NAME?"),
        // Text is not an error value.
        ("\"#VALEUR!\"&A1", "\"#VALEUR!\"&A1"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
    for (canonical, expected) in [
        ("IFERROR(#VALUE!,#N/A)", "SIERREUR(#VALEUR!;#N/A)"),
        ("{#REF!,#NUM!;#NULL!,#SPILL!}", "{#REF!.#NOMBRE!;#NUL!.#PROPAGATION!}"),
        ("#NAME?&#DIV/0!&#GETTING_DATA&#CALC!", "#NOM?&#DIV/0!&#CHARGEMENT_DONNEES&#CALC!"),
    ] {
        assert_eq!(display(canonical), expected, "{canonical}");
        assert_eq!(input(&display(canonical)), canonical, "{canonical} round trip");
    }
}

#[test]
fn french_translation_preserves_user_text_names_and_scopes() {
    for canonical in [
        "LET(SOMME,LAMBDA(x,x+1),SUM(SOMME(2),FALSE))",
        "LAMBDA(SOMME,SOMME(2))(LAMBDA(x,x+1))",
        "'SOMME;FAUX'!A1",
        "IF(A1=\"a,b;c\",CONCAT(\"SUM\",\"1.5\"),SUM(0.25,A1:A3))",
    ] {
        let expr = parse(canonical).unwrap();
        let shown = print_input(&expr, FormulaLocale::Fr, |_| false).unwrap();
        assert_eq!(print(&parse_input(&shown, FormulaLocale::Fr, |_| false).unwrap()), print(&expr), "{shown}");
    }
}
