use std::{fmt::Debug, fs::File, ops::Range, path::PathBuf, str::FromStr};

use clap::Parser;
use minghu6::{
    collections::graph::{Graph, GraphGenOptions},
    debug::graphviz::RenderOptions,
};


#[derive(Parser)]
#[clap()]
struct Args {
    /// extensions specified
    ///
    /// Candicates:
    ///
    /// + usp -- undirected graph used in shortest path
    #[clap()]
    shape: GraphShape,

    #[clap()]
    vn: usize,

    #[clap()]
    sparsity: usize,

    /// exp:
    ///
    /// + 2~4: [2, 4)
    /// + -10~5
    /// + -10~-5
    #[clap()]
    wrange: MyRange<isize>,

    #[clap(short = 'o', default_value = ".")]
    outdir: PathBuf,

    /// abc -> abc.dot && abc.csv
    #[clap()]
    r#fn: PathBuf,
}


#[derive(Clone)]
enum GraphShape {
    /// Undirected SHortest Path
    USP,
}

#[derive(Clone)]
struct MyRange<T>(Range<T>);



impl FromStr for GraphShape {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let binding = s.trim().to_lowercase();
        let s = binding.as_str();

        Ok(match s {
            "usp" => Self::USP,
            _ => return Err(format!("Unkonw ident {s}")),
        })
    }
}

impl GraphShape {
    fn get_config(&self) -> (GraphGenOptions, RenderOptions) {
        match self {
            GraphShape::USP => (
                GraphGenOptions::undir_conn(),
                RenderOptions {
                    dir: false,
                    weight_edge: true,
                },
            ),
        }
    }
}

impl<T, E> FromStr for MyRange<T>
where
    T: FromStr<Err = E>,
    E: Debug,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    where
        <T as FromStr>::Err: Debug,
    {
        let parts: Vec<&str> = s.split("~").collect();

        let fail = || format!("failed {s}");

        if parts.len() != 2 {
            return Err(fail());
        }

        let start: T = parts[0].parse().unwrap();
        let end: T = parts[1].parse().unwrap();

        Ok(Self(start..end))
    }
}




fn main() {
    let cli = Args::parse();

    let vn = cli.vn;
    let sparsity = cli.sparsity;
    let wrange = cli.wrange;
    let shape = cli.shape;

    let (gen_opt, render_opt) = shape.get_config();
    let g = Graph::gen(&gen_opt, vn, sparsity, wrange.0);

    let mut csv_fn = cli.r#fn.clone();
    csv_fn.set_extension("csv");
    let csv_path = cli.outdir.join(csv_fn);
    g.write_to_csv(&mut File::create(csv_path).unwrap())
        .unwrap();

    let mut dot_fn = cli.r#fn.clone();
    dot_fn.set_extension("dot");
    let dot_path = cli.outdir.join(dot_fn);
    g.render(&render_opt, &mut File::create(dot_path).unwrap())
        .unwrap();
}
