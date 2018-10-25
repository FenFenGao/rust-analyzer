pub(crate) mod input;

use std::{
    fmt,
    sync::Arc,
};

use ra_editor::LineIndex;
use ra_syntax::File;
use salsa;

use crate::{
    db,
    Cancelable, Canceled,
    descriptors::module::{SubmodulesQuery, ModuleTreeQuery, ModulesDatabase},
    symbol_index::SymbolIndex,
    FileId,
};

#[derive(Default)]
pub(crate) struct RootDatabase {
    runtime: salsa::Runtime<RootDatabase>,
}

impl fmt::Debug for RootDatabase {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("RootDatabase { ... }")
    }
}

impl salsa::Database for RootDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<RootDatabase> {
        &self.runtime
    }
}

pub(crate) fn check_canceled(db: &impl salsa::Database) -> Cancelable<()> {
    if db.salsa_runtime().is_current_revision_canceled() {
        Err(Canceled)
    } else {
        Ok(())
    }
}

impl salsa::ParallelDatabase for RootDatabase {
    fn fork(&self) -> Self {
        RootDatabase {
            runtime: self.runtime.fork(),
        }
    }
}

impl Clone for RootDatabase {
    fn clone(&self) -> RootDatabase {
        salsa::ParallelDatabase::fork(self)
    }
}

salsa::database_storage! {
    pub(crate) struct RootDatabaseStorage for RootDatabase {
        impl input::FilesDatabase {
            fn file_text() for input::FileTextQuery;
            fn file_source_root() for input::FileSourceRootQuery;
            fn source_root() for input::SourceRootQuery;
            fn libraries() for input::LibrarieseQuery;
            fn library_symbols() for input::LibrarySymbolsQuery;
            fn crate_graph() for input::CrateGraphQuery;
        }
        impl SyntaxDatabase {
            fn file_syntax() for FileSyntaxQuery;
            fn file_lines() for FileLinesQuery;
            fn file_symbols() for FileSymbolsQuery;
        }
        impl ModulesDatabase {
            fn module_tree() for ModuleTreeQuery;
            fn module_descriptor() for SubmodulesQuery;
        }
    }
}

salsa::query_group! {
    pub(crate) trait SyntaxDatabase: input::FilesDatabase {
        fn file_syntax(file_id: FileId) -> File {
            type FileSyntaxQuery;
        }
        fn file_lines(file_id: FileId) -> Arc<LineIndex> {
            type FileLinesQuery;
        }
        fn file_symbols(file_id: FileId) -> Cancelable<Arc<SymbolIndex>> {
            type FileSymbolsQuery;
        }
    }
}

fn file_syntax(db: &impl SyntaxDatabase, file_id: FileId) -> File {
    let text = db.file_text(file_id);
    File::parse(&*text)
}
fn file_lines(db: &impl SyntaxDatabase, file_id: FileId) -> Arc<LineIndex> {
    let text = db.file_text(file_id);
    Arc::new(LineIndex::new(&*text))
}
fn file_symbols(db: &impl SyntaxDatabase, file_id: FileId) -> Cancelable<Arc<SymbolIndex>> {
    db::check_canceled(db)?;
    let syntax = db.file_syntax(file_id);
    Ok(Arc::new(SymbolIndex::for_file(file_id, syntax)))
}
