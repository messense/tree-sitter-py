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

    fn parse(&mut self, text: &str, old_tree: Option<PyTree>) -> Option<PyTree> {
        let old_tree = old_tree.as_ref().map(|tree| &tree.inner);
        self.inner
            .parse(text, old_tree)
            .map(|inner| PyTree { inner })
    }
}

#[pymodule]
fn languages(_py: Python, m: &PyModule) -> PyResult<()> {
    let python_language = PyLanguage::from(tree_sitter_python::language());
    m.add("python", python_language)?;
    Ok(())
}

#[pyclass(module = "tree_sitter_py", name = "Tree")]
#[derive(Debug, Clone)]
struct PyTree {
    inner: tree_sitter::Tree,
}

#[pymodule]
fn tree_sitter_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLanguage>()?;
    m.add_class::<PyParser>()?;
    m.add_class::<PyTree>()?;

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
