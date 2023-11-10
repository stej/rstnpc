use hickory_resolver::config::*;
use hickory_resolver::Resolver;

pub fn resolve_record(domain: &str) -> Vec<String> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    let Ok(txt_lookup) = resolver.txt_lookup(domain) else {
        println!("No TXT record found for {}", domain);
        return Vec::new();
    };
    println!("\n{}, {:?}", domain, txt_lookup.query());
    txt_lookup.iter()
    .map(|record| {
        // fn raw_bytes_to_string(bytes: &[Box<[u8]>]) -> String {
        //     // https://stackoverflow.com/a/61861978/75224
        //     // bytes.iter()
        //     // .map(|strb| std::str::from_utf8(strb))
        //     // .try_fold(String::new(), |a, i| {
        //     //     i.map(|str| {
        //     //         let mut acc = a;
        //     //         acc.push_str(str);
        //     //         acc
        //     //     })
        //     // }).unwrap()
        //     bytes.iter()
        //     .map(|strb| std::str::from_utf8(strb).unwrap().to_string())
        //     .fold(String::new(), |acc, str| { acc+&str })
        // }
        // weird old way
        // let result_bytes = record.txt_data();
        // raw_bytes_to_string(result_bytes)
        record.to_string()
    }).collect()
}