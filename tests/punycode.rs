use rstest::rstest;

#[rstest]
#[case::egyptian("ليهمابتكلموشعربي؟")]
#[case::chinese_simplified("他们为什么不说中文")]
#[case::chinese_traditional("他們爲什麽不說中文")]
#[case::czech("Pročprostěnemluvíčesky")]
#[case::hebrew("למההםפשוטלאמדבריםעברית")]
#[case::hindi("यहलोगहिन्दीक्योंनहींबोलसकतेहैं")]
#[case::japanese("なぜみんな日本語を話してくれないのか")]
#[case::korean("세계의모든사람들이한국어를이해한다면얼마나좋을까")]
#[case::russian("почемужеонинеговорятпорусски")]
#[case::spanish("PorquénopuedensimplementehablarenEspañol")]
#[case::vietnamese("TạisaohọkhôngthểchỉnóitiếngViệt")]
#[case::kinpachi("3年B組金八先生")]
#[case::super_monkeys("安室奈美恵-with-SUPER-MONKEYS")]
#[case::hello_another_way("Hello-Another-Way-それぞれの場所")]
#[case::under_one_roof("ひとつ屋根の下2")]
#[case::takeuchi("MajiでKoiする5秒前")]
#[case::amiyumi("パフィーdeルンバ")]
#[case::at_light_speed("そのスピードで")]
#[case::money("-> $1.00 <-")]
fn encode_compare_to_punycode(#[case] input: &str) {
    let got = edna::punycode::encode(input);
    let expected = punycode::encode(input);
    assert_eq!(got.unwrap(), expected.unwrap());
}

#[rstest]
#[case::egyptian("egbpdaj6bu4bxfgehfvwxn")]
#[case::chinese_simplified("ihqwcrb4cv8a8dqg056pqjye")]
#[case::chinese_traditional("ihqwctvzc91f659drss3x8bo0yb")]
#[case::czech("Proprostnemluvesky-uyb24dma41a")]
#[case::hebrew("4dbcagdahymbxekheh6e0a7fei0b")]
#[case::hindi("i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd")]
#[case::japanese("n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa")]
#[case::korean("989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c")]
#[case::russian("b1abfaaepdrnnbgefbaDotcwatmq2g4l")]
#[case::spanish("PorqunopuedensimplementehablarenEspaol-fmd56a")]
#[case::vietnamese("TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g")]
#[case::kinpachi("3B-ww4c5e180e575a65lsy2b")]
#[case::super_monkeys("-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n")]
#[case::hello_another_way("Hello-Another-Way--fc4qua05auwb3674vfr0b")]
#[case::under_one_roof("2-u9tlzr9756bt3uc0v")]
#[case::takeuchi("MajiKoi5-783gue6qz075azm5e")]
#[case::amiyumi("de-jg4avhby1noc0d")]
#[case::at_light_speed("d9juau41awczczp")]
#[case::money("-> $1.00 <--")]
fn decode_compare_to_punycode(#[case] input: &str) {
    let got = edna::punycode::decode(input).unwrap();
    let expected = punycode::decode(input).unwrap();
    assert_eq!(got, expected);
    #[cfg(not(feature = "forbid-unsafe"))]
    let got = unsafe { edna::punycode::decode_unchecked(input) };
    assert_eq!(got, expected);
}

#[rstest]
#[case::egyptian("ليهمابتكلموشعربي؟")]
#[case::chinese_simplified("他们为什么不说中文")]
#[case::chinese_traditional("他們爲什麽不說中文")]
#[case::czech("Pročprostěnemluvíčesky")]
#[case::hebrew("למההםפשוטלאמדבריםעברית")]
#[case::hindi("यहलोगहिन्दीक्योंनहींबोलसकतेहैं")]
#[case::japanese("なぜみんな日本語を話してくれないのか")]
#[case::korean("세계의모든사람들이한국어를이해한다면얼마나좋을까")]
#[case::russian("почемужеонинеговорятпорусски")]
#[case::spanish("PorquénopuedensimplementehablarenEspañol")]
#[case::vietnamese("TạisaohọkhôngthểchỉnóitiếngViệt")]
#[case::kinpachi("3年B組金八先生")]
#[case::super_monkeys("安室奈美恵-with-SUPER-MONKEYS")]
#[case::hello_another_way("Hello-Another-Way-それぞれの場所")]
#[case::under_one_roof("ひとつ屋根の下2")]
#[case::takeuchi("MajiでKoiする5秒前")]
#[case::amiyumi("パフィーdeルンバ")]
#[case::at_light_speed("そのスピードで")]
#[case::money("-> $1.00 <-")]
fn encode_compare_to_idna(#[case] input: &str) {
    let got = edna::punycode::encode(input);
    let expected = idna::punycode::encode_str(input);
    assert_eq!(got.unwrap(), expected.unwrap());
}

#[rstest]
#[case::egyptian("egbpdaj6bu4bxfgehfvwxn")]
#[case::chinese_simplified("ihqwcrb4cv8a8dqg056pqjye")]
#[case::chinese_traditional("ihqwctvzc91f659drss3x8bo0yb")]
#[case::czech("Proprostnemluvesky-uyb24dma41a")]
#[case::hebrew("4dbcagdahymbxekheh6e0a7fei0b")]
#[case::hindi("i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd")]
#[case::japanese("n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa")]
#[case::korean("989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c")]
#[case::russian("b1abfaaepdrnnbgefbaDotcwatmq2g4l")]
#[case::spanish("PorqunopuedensimplementehablarenEspaol-fmd56a")]
#[case::vietnamese("TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g")]
#[case::kinpachi("3B-ww4c5e180e575a65lsy2b")]
#[case::super_monkeys("-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n")]
#[case::hello_another_way("Hello-Another-Way--fc4qua05auwb3674vfr0b")]
#[case::under_one_roof("2-u9tlzr9756bt3uc0v")]
#[case::takeuchi("MajiKoi5-783gue6qz075azm5e")]
#[case::amiyumi("de-jg4avhby1noc0d")]
#[case::at_light_speed("d9juau41awczczp")]
#[case::money("-> $1.00 <--")]
fn decode_compare_to_idna(#[case] input: &str) {
    let got = edna::punycode::decode(input).unwrap();
    let expected = idna::punycode::decode_to_string(input).unwrap();
    assert_eq!(got, expected);
    #[cfg(not(feature = "forbid-unsafe"))]
    let got = unsafe { edna::punycode::decode_unchecked(input) };
    assert_eq!(got, expected);
}
