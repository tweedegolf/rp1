use proc_macro2::TokenStream;

pub enum Error {
    SynError(syn::Error),
    Darling(darling::Error),
    MissingPrimaryKey,
    AggregatePrimaryKeyNotSupported,
    UnnamedFieldsNotSupported,
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Self {
        Error::SynError(e)
    }
}

impl From<darling::Error> for Error {
    fn from(e: darling::Error) -> Self {
        Error::Darling(e)
    }
}

impl Error {
    pub fn into_compile_error(self) -> TokenStream {
        match self {
            Error::SynError(e) => e.to_compile_error(),
            Error::Darling(e) => e.write_errors(),
            Error::MissingPrimaryKey => todo!(),
            Error::AggregatePrimaryKeyNotSupported => todo!(),
            Error::UnnamedFieldsNotSupported => todo!(),
        }
    }
}

pub type Result<T = TokenStream, E = Error> = std::result::Result<T, E>;
