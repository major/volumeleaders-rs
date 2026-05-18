use super::filters::set_filter;

#[derive(Debug)]
pub(super) struct TradePreset {
    pub(super) name: &'static str,
    pub(super) group: &'static str,
    base: PresetBase,
    filters: &'static [(&'static str, &'static str)],
}

#[derive(Clone, Copy, Debug)]
enum PresetBase {
    Common,
    Large,
    None,
}

static TRADE_PRESETS: &[TradePreset] = &[
    TradePreset {
        name: "All Trades",
        group: "Common",
        base: PresetBase::Common,
        filters: &[],
    },
    TradePreset {
        name: "Top-10 Rank",
        group: "Common",
        base: PresetBase::Common,
        filters: &[("TradeRank", "10")],
    },
    TradePreset {
        name: "Top-100 Rank",
        group: "Common",
        base: PresetBase::Common,
        filters: &[("MaxDollars", "100000000000"), ("TradeRank", "100")],
    },
    TradePreset {
        name: "Top-100 Rank; Dark Pool Sweeps",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "-1"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("MaxDollars", "100000000000"),
            ("MinVolume", "10000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("Sweeps", "1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; Leveraged ETFs",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("MaxDollars", "1000000000000"),
            ("SectorIndustry", "X B"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; RSI OB; >=5x Avg Size",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "OBD,OBH"),
            ("IncludeOffsetting", "-1"),
            ("IncludePhantom", "-1"),
            ("MaxDollars", "10000000000"),
            ("MinVolume", "10000"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; RSI OS; >=5x Avg Size",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "OSD,OSH"),
            ("IncludeOffsetting", "-1"),
            ("IncludePhantom", "-1"),
            ("MaxDollars", "10000000000"),
            ("MinVolume", "10000"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; >=20x avg size; DP Only",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("DarkPools", "1"),
            ("RelativeSize", "20"),
            ("SignaturePrints", "0"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-30 Rank; >10x avg size; 99th %",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("RelativeSize", "10"),
            ("SignaturePrints", "0"),
            ("TradeRank", "30"),
            ("VCD", "99.00"),
        ],
    },
    TradePreset {
        name: "Phantom Trades",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "0"),
            ("IncludeOpening", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
        ],
    },
    TradePreset {
        name: "Offsetting Trades",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
        ],
    },
    TradePreset {
        name: "All Disproportionately Large Trades",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[],
    },
    TradePreset {
        name: "Bear Leverage",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "X Bear"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "Biotechnology",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Biotech")],
    },
    TradePreset {
        name: "Bonds",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Bonds")],
    },
    TradePreset {
        name: "Bull Leverage",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "X Bull"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "China",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "China"), ("MaxDollars", "100000000000")],
    },
    TradePreset {
        name: "Communication Services",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Comm Services")],
    },
    TradePreset {
        name: "Consumer Discretionary",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Consumer Disc")],
    },
    TradePreset {
        name: "Consumer Staples",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Consumer Staples")],
    },
    TradePreset {
        name: "Crypto",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Crypto"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "Emerging Markets",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Emerging Markets")],
    },
    TradePreset {
        name: "Energy",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Energy")],
    },
    TradePreset {
        name: "Financials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Financial")],
    },
    TradePreset {
        name: "Healthcare",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Healthcare")],
    },
    TradePreset {
        name: "Industrials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Industrials")],
    },
    TradePreset {
        name: "Materials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Materials")],
    },
    TradePreset {
        name: "Metals and Mining",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Metals and Mining")],
    },
    TradePreset {
        name: "Real Estate",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Real Estate")],
    },
    TradePreset {
        name: "Semiconductors",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Semis")],
    },
    TradePreset {
        name: "Technology",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Technology")],
    },
    TradePreset {
        name: "Utilities",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Utilities")],
    },
    TradePreset {
        name: "Commodities",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "AGQ,BOIL,CORN,COPX,CPER,DBC,DJP,GLD,GLDM,IAU,KOLD,PPLT,SCO,SLV,SOYB,UCO,UGL,UNG,URA,USO,UUP,WEAT,ZSL",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Electric Vehicles",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "BLNK,F,GM,LI,NIO,NKLA,TSLA,WKHS,QS,LCID,RIVN,TSLQ,TSLL,TSLS,TSLY,TSDD",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Megacaps",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            ("Tickers", "AAPL,AMZN,META,GOOG,GOOGL,MSFT,NFLX,NVDA,TSLA"),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Meme Stocks",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "AMC,BB,CLF,GME,NOK,SAVA,SPCE,TLRY,LOGC,CLOV,SOFI,BKKT,PUBM",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Sector ETFs",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "DGRO,EEM,GLD,IBB,ITOT,IVE,IVW,IVV,IWM,IWY,MDY,QQQ,RSP,SLV,SMH,SPYD,SPY,SPYV,SPYG,TLT,USO,XBI,XLE,XLK,XLP,XLI,XLF,XLC,XLY,XLV,XLU",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "SPY/QQQ Surrogates",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "ACWI,DGRO,FBCG,FBCV,IWL,IWB,IVW,IVV,IWF,IWX,IWV,IWY,MGC,MGK,MGV,MTUM,OEF,PSQ,QLD,QID,QQQE,QQQ,QQEW,RSP,SCHG,SCHK,SCHV,SCHX,SDS,SH,SPYM,SPXS,SPXL,SPYD,SPY,SQQQ,SPYV,SPXU,SPYG,SSO,SUSA,TCHP,TQQQ,UDOW,UPRO,VFVA,VOO,VOOG,VOOV,VUG,VV,VTV,XLK,CGGR,JGRO,SPYU",
            ),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
        ],
    },
    TradePreset {
        name: "Volatility",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            ("Tickers", "SVXY,UVXY,VIXY,VXX,SVIX,UVIX"),
            ("VCD", "97.00"),
        ],
    },
];

pub(super) fn find_trade_preset(name: &str) -> Option<&'static TradePreset> {
    TRADE_PRESETS
        .iter()
        .find(|preset| preset.name.eq_ignore_ascii_case(name))
}

pub(super) fn apply_preset_filters(filters: &mut Vec<(String, String)>, preset: &TradePreset) {
    let _ = preset.group;
    match preset.base {
        PresetBase::Common => apply_common_preset_filters(filters),
        PresetBase::Large => apply_large_preset_filters(filters),
        PresetBase::None => {}
    }
    for &(key, value) in preset.filters {
        set_filter(filters, key, value.to_string());
    }
}

fn apply_common_preset_filters(filters: &mut Vec<(String, String)>) {
    for (key, value) in [
        ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
        ("IncludeOffsetting", "-1"),
        ("IncludePhantom", "-1"),
        ("MaxDollars", "10000000000"),
        ("MinVolume", "10000"),
        ("RelativeSize", "0"),
        ("TradeCount", "3"),
    ] {
        set_filter(filters, key, value.to_string());
    }
}

fn apply_large_preset_filters(filters: &mut Vec<(String, String)>) {
    for (key, value) in [
        ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
        ("IncludeOffsetting", "-1"),
        ("IncludePhantom", "-1"),
        ("MaxDollars", "10000000000"),
        ("MinVolume", "10000"),
        ("TradeCount", "3"),
    ] {
        set_filter(filters, key, value.to_string());
    }
}
