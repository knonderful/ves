use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

struct RomDataEntrySpec {
    id: String,
    path: PathBuf,
    byte_count: usize,
}

impl RomDataEntrySpec {
    fn new(id: &str, path: &str) -> Self {
        let path = PathBuf::from(path);
        let byte_count = fs::metadata(&path)
            .map_err(|e| format!("Could not obtain metadata for {}: {}", path.to_str().unwrap(), e.to_string()))
            .unwrap()
            .len() as usize;

        RomDataEntrySpec {
            id: id.into(),
            path,
            byte_count,
        }
    }
}

struct TemplateWriter {
    file: File,
}

impl TemplateWriter {
    fn struct_start(&mut self, name: &str) {
        self.raw_line("#[repr(C)]");
        self.raw(format!("pub struct {} {{", name).as_str());
    }

    fn struct_end(&mut self) {
        self.raw_line("");
        self.raw_line("}");
        self.raw_line("");
    }

    fn field(&mut self, name: &str, ty: &str) {
        self.raw(format!("\n    {}: {},", name, ty).as_str());
    }

    fn raw(&mut self, string: &str) {
        self.file.write_all(string.as_bytes()).unwrap();
    }

    fn raw_line(&mut self, string: &str) {
        self.raw(string);
        self.raw("\n");
    }
}

struct RomDataSpec {
    struct_name: &'static str,
    version: (u16, u16, u16),
    gfx: Vec<RomDataEntrySpec>,
}

macro_rules! entry_vec {
    ( $( ($field:expr, $path:expr) ),* ) => {
        {
            vec!(
                $(
                    RomDataEntrySpec::new($field, $path),
                )*
            )
        }
    };
}

fn main() {
    let params = RomDataSpec {
        struct_name: "RomData",
        version: (0, 1, 16),
        gfx: entry_vec!(
            ("mario", "assets/gfx/mario.png"),
            ("mario2", "assets/gfx/mario.png")
        ),
    };

    let mut template = create_file().unwrap();

    template.struct_start("RomDataEntry<T>");
    template.field("data", "T");
    template.struct_end();

    template.raw_line("impl<T> RomDataEntry<T> {");
    template.raw_line("    pub const fn new(data: T) -> Self {");
    template.raw_line("        RomDataEntry { data }");
    template.raw_line("    }");
    template.raw_line("}");
    template.raw_line("");

    template.struct_start("RomDataGfx");
    for RomDataEntrySpec { id, path: _, byte_count } in &params.gfx {
        template.field(format!("pub {}", id).as_str(), format!("RomDataEntry<[u8; {}usize]>", byte_count).as_str());
    }
    template.struct_end();

    template.struct_start(params.struct_name);
    template.field("pub version", "(u16, u16, u16)");
    template.field("pub gfx", "RomDataGfx");
    template.struct_end();

    template.raw_line("macro_rules! insert_rom_data {");
    template.raw_line("    () => {");
    template.raw_line(format!("        {} {{", params.struct_name).as_str());
    template.raw_line(format!("            version: {:?},", params.version).as_str());
    template.raw_line("            gfx: RomDataGfx {");
    for RomDataEntrySpec { id, path, byte_count: _ } in &params.gfx {
        template.raw_line(format!("                {}: RomDataEntry::new(*include_bytes!(\"../{}\")),", id, path.to_str().unwrap()).as_str());
    }
    template.raw_line("            },");
    template.raw_line("        }");
    template.raw_line("    };");
    template.raw_line("}");

    println!("cargo:rerun-if-changed=build.rs");
}

fn create_file() -> std::io::Result<TemplateWriter> {
    let dest_path = Path::new("src/rom_data.rs");
    Ok(TemplateWriter { file: File::create(dest_path)? })
}
