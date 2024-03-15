mod errors;
mod iter;
mod preprocessor;
mod store;

// The preprocessor does only resolve how many arguments \newcommand and
// \newenv commands take. Previous implementations also resolved \input and
// \usepackage, what is now delegated to macro expansion and the function call
// process.
