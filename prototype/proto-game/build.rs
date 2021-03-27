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

const FN_CALC_ROM_DATA_OFFSET: &'static str = "calc_rom_data_offset";
const TYPE_ROM_DATA_POINTER: &'static str = "u32";
const TYPE_ROM_BLOCK: &'static str = "proto_common::mem::RomBlock";
const TYPE_ROM_DATA_ENTRY: &'static str = "RomDataEntry";
const TYPE_ROM_DATA_GFX: &'static str = "RomDataGfx";
const TYPE_ROM_DATA: &'static str = "RomData";
const FIELD_ROM_DATA_GFX: &'static str = "gfx";

fn main() {
    let params = RomDataSpec {
        version: (0, 1, 16),
        gfx: entry_vec!(
            ("example", "assets/gfx/example.data")
        ),
    };

    let mut template = create_file().unwrap();

    template.raw_line("#[inline(always)]");
    template.raw_line("fn rom_data() -> &'static RomData");
    template.raw_line("{");
    template.raw_line("    unsafe {");
    template.raw_line("        let null = std::ptr::null();");
    template.raw_line("        &(*null)");
    template.raw_line("    }");
    template.raw_line("}");
    template.raw_line("");

    template.raw_line("#[inline(always)]");
    template.raw_line(format!("fn {}<F, T>(func: F) -> {}", FN_CALC_ROM_DATA_OFFSET, TYPE_ROM_DATA_POINTER).as_str());
    template.raw_line("    where F: FnOnce(&RomData) -> &T");
    template.raw_line("{");
    template.raw_line(format!("    func(rom_data()) as *const T as {}", TYPE_ROM_DATA_POINTER).as_str());
    template.raw_line("}");
    template.raw_line("");

    template.struct_start(format!("{}<T>", TYPE_ROM_DATA_ENTRY).as_str());
    template.field("data", "T");
    template.struct_end();

    template.raw_line(format!("impl<T> {}<T> {{", TYPE_ROM_DATA_ENTRY).as_str());
    template.raw_line("    const fn new(data: T) -> Self {");
    template.raw_line("        Self { data }");
    template.raw_line("    }");
    template.raw_line("}");
    template.raw_line("");


    template.struct_start(TYPE_ROM_DATA_GFX);
    for RomDataEntrySpec { id, path: _, byte_count } in &params.gfx {
        template.field(id, format!("{}<[u8; {}usize]>", TYPE_ROM_DATA_ENTRY, byte_count).as_str());
    }
    template.struct_end();

    template.raw_line(format!("impl {} {{", TYPE_ROM_DATA_GFX).as_str());
    for RomDataEntrySpec { id, path: _, byte_count } in &params.gfx {
        template.raw_line(format!("    pub fn {}(&self) -> {} {{", id, TYPE_ROM_BLOCK).as_str());
        template.raw_line(format!("        {}::new(", TYPE_ROM_BLOCK).as_str());
        template.raw_line(format!("            {}(|rom| &rom.{}.{}),", FN_CALC_ROM_DATA_OFFSET, FIELD_ROM_DATA_GFX, id).as_str());
        template.raw_line(format!("            {}", byte_count).as_str());
        template.raw_line("        )");
        template.raw_line("    }");
        template.raw_line("");
    }
    template.raw_line("}");
    template.raw_line("");

    template.struct_start(TYPE_ROM_DATA);
    template.field("version", "(u16, u16, u16)");
    template.field(FIELD_ROM_DATA_GFX, TYPE_ROM_DATA_GFX);
    template.struct_end();

    template.raw_line(format!("impl {} {{", TYPE_ROM_DATA).as_str());
    template.raw_line(format!("    pub const fn create() -> {} {{", TYPE_ROM_DATA).as_str());
    template.raw_line(format!("        {} {{", TYPE_ROM_DATA).as_str());
    template.raw_line(format!("            version: {:?},", params.version).as_str());
    template.raw_line(format!("            {}: {} {{", FIELD_ROM_DATA_GFX, TYPE_ROM_DATA_GFX).as_str());
    for RomDataEntrySpec { id, path, byte_count: _ } in &params.gfx {
        template.raw_line(format!("                {}: {}::new(*include_bytes!(\"../{}\")),", id, TYPE_ROM_DATA_ENTRY, path.to_str().unwrap()).as_str());
    }
    template.raw_line("            },");
    template.raw_line("        }");
    template.raw_line("    }");
    template.raw_line("    ");
    template.raw_line(format!("    pub fn {}(&self) -> &{} {{", FIELD_ROM_DATA_GFX, TYPE_ROM_DATA_GFX).as_str());
    template.raw_line(format!("        &self.{}", FIELD_ROM_DATA_GFX).as_str());
    template.raw_line("    }");
    template.raw_line("}");

    println!("cargo:rerun-if-changed=build.rs");
}

fn create_file() -> std::io::Result<TemplateWriter> {
    let dest_path = Path::new("src/rom_data.rs");
    Ok(TemplateWriter { file: File::create(dest_path)? })
}
