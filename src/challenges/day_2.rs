use actix_web::{web, Scope};

mod ipv4 {
    use std::net::Ipv4Addr;

    use actix_web::{get, web};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct SantaFromKey {
        from: Ipv4Addr,
        key: Ipv4Addr
    }
    impl SantaFromKey {
        fn dest(&self) -> String {
            self.from.octets().iter().zip(&self.key.octets()).map(|(f, k)| {
                f.wrapping_add(*k).to_string()
            }).collect::<Vec<String>>().join(".")
        }
    }

    #[get("/dest")]
    async fn dest(route: web::Query<SantaFromKey>) -> String {
        route.dest()
    }

    #[derive(Deserialize)]
    struct SantaFromTo {
        from: Ipv4Addr,
        to: Ipv4Addr
    }
    impl SantaFromTo {
        fn key(&self) -> String {
            self.from.octets().iter().zip(&self.to.octets()).map(|(f, t)| {
                t.wrapping_sub(*f).to_string()
            }).collect::<Vec<String>>().join(".")
        }
    }

    #[get("/key")]
    async fn key(route: web::Query<SantaFromTo>) -> String {
        route.key()
    }
}

mod ipv6 {
    use std::net::Ipv6Addr;

    use actix_web::{get, web};
    use serde::Deserialize;

    fn build(l: Ipv6Addr, r: Ipv6Addr) -> String {
            Ipv6Addr::from(
                TryInto::<[u16;8]>::try_into(
                l.segments().iter().zip(r.segments()).map(|(l, r)| {
                        l ^ r
                    }).collect::<Vec<u16>>()
                ).unwrap()
            ).to_string()
    }

    #[derive(Deserialize)]
    struct SantaFromKey {
        from: Ipv6Addr,
        key: Ipv6Addr
    }
    #[get("/v6/dest")]
    async fn dest(route: web::Query<SantaFromKey>) -> String {
        build(route.from, route.key)
    }

    #[derive(Deserialize)]
    struct SantaFromTo {
        from: Ipv6Addr,
        to: Ipv6Addr
    }
    #[get("/v6/key")]
    async fn key(route: web::Query<SantaFromTo>) -> String {
        build(route.from, route.to)
    }
}

pub fn scope() -> Scope {
    web::scope("/2")
        .service(ipv4::dest)
        .service(ipv4::key)
        .service(ipv6::dest)
        .service(ipv6::key)
}