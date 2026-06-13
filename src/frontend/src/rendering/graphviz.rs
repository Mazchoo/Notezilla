use layout::backends::svg::SVGWriter;
use layout::gv::{DotParser, GraphBuilder};

pub fn render_dot(dot: &str) -> Result<String, String> {
    let mut parser = DotParser::new(dot);
    let tree = parser.process().map_err(|e| format!("{e:?}"))?;
    let mut builder = GraphBuilder::new();
    builder.visit_graph(&tree);
    let mut vg = builder.get();
    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);
    Ok(svg.finalize())
}
