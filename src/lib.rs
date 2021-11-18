use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

#[pyclass(module = "tree_sitter_py", name = "Language")]
#[derive(Debug, Clone)]
struct PyLanguage {
    inner: tree_sitter::Language,
}

#[pymethods]
impl PyLanguage {
    fn version(&self) -> usize {
        self.inner.version()
    }
}

impl From<tree_sitter::Language> for PyLanguage {
    fn from(lang: tree_sitter::Language) -> Self {
        Self { inner: lang }
    }
}

#[pyclass(module = "tree_sitter_py", name = "Parser")]
struct PyParser {
    inner: tree_sitter::Parser,
}

#[pymethods]
impl PyParser {
    #[new]
    fn new() -> Self {
        Self {
            inner: tree_sitter::Parser::new(),
        }
    }

    fn set_language(&mut self, language: PyLanguage) -> PyResult<()> {
        self.inner.set_language(language.inner).unwrap();
        Ok(())
    }

    fn language(&self) -> Option<PyLanguage> {
        self.inner.language().map(|lang| lang.into())
    }

    #[args(old_tree = "None")]
    fn parse(&mut self, text: &str, old_tree: Option<PyTree>) -> Option<PyTree> {
        let old_tree = old_tree.as_ref().map(|tree| &*tree.inner);
        self.inner.parse(text, old_tree).map(|inner| PyTree {
            inner: Arc::new(inner),
        })
    }
}

#[pyclass(module = "tree_sitter_py", name = "Tree")]
#[derive(Debug, Clone)]
struct PyTree {
    inner: Arc<tree_sitter::Tree>,
}

#[pymethods]
impl PyTree {
    fn root_node(&self) -> PyNode {
        PyNode::new(Arc::clone(&self.inner), |tree| tree.root_node())
    }

    fn walk(&self) -> PyTreeCursor {
        PyTreeCursor::new(Arc::clone(&self.inner), |tree| tree.walk())
    }
}

#[pyclass(module = "tree_sitter_py", name = "Node", unsendable)]
#[ouroboros::self_referencing]
#[derive(Debug)]
struct PyNode {
    tree: Arc<tree_sitter::Tree>,
    #[borrows(tree)]
    #[covariant]
    node: tree_sitter::Node<'this>,
}

#[pymethods]
impl PyNode {
    fn id(&self) -> usize {
        self.borrow_node().id()
    }

    fn kind_id(&self) -> u16 {
        self.borrow_node().kind_id()
    }

    fn kind(&self) -> &'static str {
        self.borrow_node().kind()
    }

    fn language(&self) -> PyLanguage {
        self.borrow_node().language().into()
    }
}

#[pyclass(module = "tree_sitter_py", name = "TreeCursor", unsendable)]
#[ouroboros::self_referencing]
struct PyTreeCursor {
    tree: Arc<tree_sitter::Tree>,
    #[borrows(tree)]
    #[covariant]
    cursor: tree_sitter::TreeCursor<'this>,
}

#[pymethods]
impl PyTreeCursor {
    fn field_id(&self) -> Option<u16> {
        self.borrow_cursor().field_id()
    }

    fn field_name(&self) -> Option<&'static str> {
        self.borrow_cursor().field_name()
    }
}

#[pymodule]
fn languages(_py: Python, m: &PyModule) -> PyResult<()> {
    let python = PyLanguage::from(tree_sitter_python::language());
    m.add("python", python)?;

    let typescript = PyLanguage::from(tree_sitter_typescript::language_typescript());
    m.add("typescript", typescript)?;
    let tsx = PyLanguage::from(tree_sitter_typescript::language_tsx());
    m.add("tsx", tsx)?;
    Ok(())
}

#[pymodule]
fn tree_sitter_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLanguage>()?;
    m.add_class::<PyParser>()?;
    m.add_class::<PyTree>()?;
    m.add_class::<PyNode>()?;

    m.add_wrapped(wrap_pymodule!(languages))?;

    py.run(
        "\
import sys
sys.modules['tree_sitter_py.languages'] = languages
    ",
        None,
        Some(m.dict()),
    )?;
    Ok(())
}
