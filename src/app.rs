use std::{error, fmt};
use leptos::prelude::*;
use leptos::html::Input;
use leptos_meta::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;
use num_format::{Locale, ToFormattedString};
use thaw::*;
use uibooks::{Record, SectionStats, process_records};
use wasm_bindgen_futures::JsFuture;
use web_sys::MouseEvent;

// App component

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Stylesheet id="leptos" href="/style/output.css"/>
        <leptos_meta::Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Title formatter=|text| format!("{text} - Reading Wrapped") />
        
        <ConfigProvider>
            <Router>
                <main class="bg-gradient-to-br from-amber-50 via-orange-50/40 to-stone-100 min-h-screen text-slate-800 flex flex-col justify-between selection:bg-amber-200 selection:text-amber-900">
                    <div class="max-w-4xl mx-auto w-full p-4 sm:p-6 md:p-8 flex-grow">
                        <Routes fallback=|| view! { <NotFound/> }>
                            <Route path=path!("/") view=Home/>
                            <Route path=path!("/*any") view=|| view! { <NotFound/> }/>
                        </Routes>
                    </div>
                    
                    <footer class="py-6 text-center text-xs text-slate-500 border-t border-amber-900/10 bg-white/40 backdrop-blur-sm space-y-1">
                        <p>"Reading Wrapped • Built with Leptos, Thaw UI & Tailwind CSS"</p>
                        <p class="text-slate-400">"All processing happens locally in your browser—no data is stored or collected."</p>
                        <p class="text-[11px] text-slate-400/80">"Not affiliated with or endorsed by Goodreads."</p>
                    </footer>
                </main>
            </Router>
        </ConfigProvider>
    }
}

// Not found component

#[component]
fn NotFound() -> impl IntoView {
    view!{
        <div class="mt-12">
            <SectionCard
                title=format!("Uh Oh!")
                caption=format!("Page not found")
            >
                <SectionSlot slot>
                    <p class="text-slate-600 py-2">"It looks like we are not able to find that page."</p>
                </SectionSlot>
            </SectionCard>
        </div>
    }
}

// Home component

#[component]
fn Home() -> impl IntoView {
    let (show_wrapped, set_show_wrapped) = signal(false);
    let (csv_data, set_csv_data) = signal(None::<Vec<Record>>);
    provide_context(set_show_wrapped);
    provide_context(csv_data);
    provide_context(set_csv_data);

    view!{
        <Title text="Home" />
        <div class="transition-all duration-300">
            <Show
                when=move || show_wrapped.get()
                fallback=move || {
                    view! {
                        <Upload/>
                    }
                }
            >
                <Wrapped/>
            </Show>
        </div>
    }
}

// Upload component

#[derive(Debug, Clone)]
enum UploadError {
    InternalError,
    NoFileSelected,
    FileReadError,
    InvalidStructure(String)
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UploadError::InternalError => write!(f, "An internal error has occurred."),
            UploadError::NoFileSelected => write!(f, "No file was selected."),
            UploadError::FileReadError => write!(f, "Failed to read the file data."),
            UploadError::InvalidStructure(details) => write!(f, "Invalid CSV structure or headers: {}", details)
        }
    }
}

impl error::Error for UploadError {}

#[component]
fn Upload() -> impl IntoView {
    let input_element = NodeRef::<Input>::new();
    let (file_name, set_file_name) = signal(None::<String>);

    let on_file_change = move |_| {
        if let Some(file) = input_element
            .get()
            .and_then(|input| input.files())
            .and_then(|files| files.get(0)) {
                set_file_name.set(Some(file.name()));
        } else {
            set_file_name.set(None);
        }
    };

    let upload_action = Action::new_local(move |_: &()| {
        let input_element = input_element;
        async move {
            let show_wrapped_setter = use_context::<WriteSignal<bool>>().ok_or(UploadError::InternalError)?;
            let records_setter = use_context::<WriteSignal<Option<Vec<Record>>>>().ok_or(UploadError::InternalError)?;
            records_setter.set(Some(Vec::new()));

            let input = input_element.get().ok_or(UploadError::InternalError)?;
            let files = input.files().ok_or(UploadError::NoFileSelected)?;
            let file = files.get(0).ok_or(UploadError::NoFileSelected)?;

            let file_promise = file.text();
            let file_value = JsFuture::from(file_promise).await.map_err(|_| UploadError::FileReadError)?;
            let text = file_value.as_string().ok_or(UploadError::FileReadError)?;

            let mut reader = csv::Reader::from_reader(text.as_bytes());

            let mut records = Vec::new();
            let mut errors = Vec::new();
            for (index, result) in reader.deserialize::<Record>().enumerate() {
                let line_number = index + 2;
                match result {
                    Ok(record) => records.push(record),
                    Err(err) => errors.push((line_number, err.to_string()))
                }
            }

            if records.is_empty() && !errors.is_empty() {
                return Err(UploadError::InvalidStructure("Unable to parse any rows. Please ensure your csv structure is correct.".to_string()));
            }

            records_setter.set(Some(records));
            show_wrapped_setter.set(true);

            Ok(errors)
        }
    });

    let on_submit = move |ev: MouseEvent| {
        ev.prevent_default();
        upload_action.dispatch(());
    };

    let value = upload_action.value();
    let pending = upload_action.pending();

    view!{
        <div class="max-w-xl mx-auto mt-10">
            <SectionCard
                title=format!("Welcome to your reading wrapped!")
                caption=format!("In order to view information about your reading habits, please upload a csv containing your reading information.")
            >
                <SectionSlot slot>
                    <div class="space-y-6">
                        <label class="flex flex-col items-center justify-center w-full h-40 p-6 border-2 border-dashed border-amber-300 rounded-2xl bg-white/60 hover:border-amber-500 hover:bg-amber-50/50 cursor-pointer transition-all duration-200 shadow-sm">
                            <input on:change=on_file_change type="file" accept=".csv" node_ref=input_element class="hidden"/>
                            <div class="flex flex-col items-center justify-center space-y-2 text-center">
                                <svg class="w-8 h-8 text-amber-600 mb-1 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                                </svg>
                                <span class="text-sm font-semibold text-slate-700">
                                    "Click or drag a csv file to this area to upload"
                                </span>
                                <span class="text-xs text-slate-400">"Supports standard Goodreads CSV exports"</span>
                            </div>
                        </label>

                        <Show when=move || file_name.get().is_some() fallback=move || ()>
                            <div class="flex items-center space-x-2 text-xs font-medium text-slate-600 bg-amber-50 border border-amber-200 px-4 py-2.5 rounded-xl">
                                <svg class="w-4 h-4 text-amber-600 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                </svg>
                                <span class="truncate">
                                    "Selected file: " <strong class="text-slate-800">{move || file_name.get().unwrap_or_default()}</strong>
                                </span>
                            </div>
                        </Show>

                        <Show when=move || pending.get()>
                            <div class="flex items-center justify-center space-x-2 text-amber-700 font-medium py-2">
                                <svg class="animate-spin -ml-1 mr-3 h-5 text-amber-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                <span>"Processing CSV..."</span>
                            </div>
                        </Show>

                        <ErrorBoundary fallback=move |errors| view! {
                            <div class="bg-red-50 border border-red-200 rounded-xl p-4 text-red-700 space-y-2">
                                <h3 class="font-semibold text-sm">"Upload Failed"</h3>
                                <ul class="text-xs space-y-1 list-disc list-inside">
                                    {move || errors.get().into_iter().map(|(_, e)| view! { <li>{e.to_string()}</li> }).collect::<Vec<_>>()}
                                </ul>
                            </div>
                        }>
                            {move || {
                                value.get().map(|respose| {
                                    respose.map(|errors| {
                                        let has_errors = !errors.is_empty();
                                        view! {
                                            <Show
                                                when=move || has_errors
                                                fallback=move || ()
                                            >
                                                <div class="bg-amber-50 border border-amber-200 rounded-xl p-4 text-amber-800 space-y-2">
                                                    <h4 class="font-semibold text-sm">"Row Parsing Warnings/Errors:"</h4>
                                                    <ul class="text-xs space-y-1 max-h-32 overflow-y-auto">
                                                        {errors.iter().map(|(line, err_msg)| {
                                                            view! {
                                                                <li class="py-0.5">
                                                                    <strong class="font-medium">"Line " {*line} ": "</strong>
                                                                    {err_msg.clone()}
                                                                </li>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </ul>
                                                </div>
                                            </Show>
                                        }
                                    })
                                })
                            }}
                        </ErrorBoundary>

                        <CardFooter class="pt-2 flex justify-end">
                            <Button on_click=on_submit disabled=pending class="bg-amber-600 hover:bg-amber-700 text-white font-medium px-6 py-2 rounded-xl shadow-sm transition-all">"Submit"</Button>
                        </CardFooter>
                    </div>
                </SectionSlot>
            </SectionCard>
        </div>
    }
}

// Wrapped component

#[component]
fn Wrapped() -> impl IntoView {
    let show_wrapped_setter = use_context::<WriteSignal<bool>>();
    let handle_click = move |_| {
        if let Some(show_wrapped_setter) = show_wrapped_setter {
            show_wrapped_setter.set(false);
        }
    };

    let records = move || {
        use_context::<ReadSignal<Option<Vec<Record>>>>()
            .map(|sig| sig.get())
            .flatten()
    };

    let stats = move || {
        records().and_then(|rec| process_records(rec).ok())
    };

    let rating_breakdown = move || {
        stats()
            .as_ref()
            .map(|stat| {
                stat.rating_section.rating_breakdown
                    .iter()
                    .map(|(rating, amount)| {
                        view! { 
                            <li class="text-sm text-slate-600 flex justify-between items-center py-1.5 border-b border-slate-200/60 last:border-none">
                                <span class="text-left">{format!("{} stars", rating)}</span> 
                                <span class="font-semibold text-slate-800 text-right">{format!("{} books", amount)}</span>
                            </li> 
                        }
                    }).collect_view()
            })
    };

    let author_breakdown = move || {
        stats()
            .as_ref()
            .and_then(|stat| stat.author_section.author_breakdown.get(0..5))
            .map(|authors| {
                authors.iter()
                    .map(|(author, count)| {
                        view! { 
                            <li class="text-sm text-slate-600 flex justify-between items-center py-1.5 border-b border-slate-200/60 last:border-none">
                                <span class="text-left truncate pr-2">{author.clone()}</span> 
                                <span class="font-semibold text-slate-800 text-right shrink-0">{format!("{} books", count)}</span>
                            </li> 
                        }
                    })
                    .collect_view()
            })
    };

    let months_breakdown = move || {
        stats()
            .as_ref()
            .map(|stat| {
                stat.speed_section.months_breakdown
                    .iter()
                    .map(|(month, amount)| {
                        view! { 
                            <li class="text-sm text-slate-600 flex justify-between items-center py-1.5 border-b border-slate-200/60 last:border-none">
                                <span class="text-left">{month.clone()}</span> 
                                <span class="font-semibold text-slate-800 text-right">{format!("{} books", amount)}</span>
                            </li> 
                        }
                    }).collect_view()
            })
    };

    view! {
        <div class="space-y-6">
            <Card class="bg-white/80 backdrop-blur-md shadow-lg rounded-2xl border border-amber-900/10 p-2">
                <CardHeader class="flex flex-row items-center justify-between pb-4 border-b border-slate-100">
                    <div>
                        <Body1 class="text-xl font-bold text-slate-900">
                            <b>"Reading Wrapped"</b>
                        </Body1>
                        <CardHeaderDescription slot>
                            <Caption1 class="text-slate-500 text-xs mt-0.5">{format!("See information about your reading!")}</Caption1>
                        </CardHeaderDescription>
                    </div>
                    <CardHeaderAction slot>
                        <Button
                            on_click=handle_click
                            class="text-sm font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 px-4 py-2 rounded-xl transition-all"
                        >
                            "Back"
                        </Button>
                    </CardHeaderAction>
                </CardHeader>
                <div class="pt-4">
                    <Show
                        when=move || records().is_some()
                        fallback=move || {
                            view! {
                                <SectionCard
                                    title=format!("Reading Wrapped")
                                    caption=format!("See a summary of your reading!")
                                >
                                    <SectionSlot slot>
                                        <p class="text-slate-500 py-4">"No book data uploaded yet."</p>
                                    </SectionSlot>
                                </SectionCard>
                            }
                        }
                    >
                        <div class="space-y-6">
                            {move || {
                                match stats() {
                                    Some(SectionStats { rating_section, page_section, author_section, speed_section }) => {
                                        view! {
                                            <div class="space-y-6">
                                                <SectionCard
                                                    title=format!("Rating Wrapped")
                                                    caption=format!("See how your ratings checked out!")
                                                >
                                                    <SectionSlot slot>
                                                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-2">
                                                            <SectionInnerCard
                                                                title=format!("Average Rating")
                                                                value=format!("{:.2} stars", rating_section.average_rating)
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Top Rating")
                                                                value=format!("{} stars", rating_section.top_rating.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Lowest Rating")
                                                                value=format!("{} stars", rating_section.low_rating.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Rating Breakdown")
                                                            >
                                                                <SectionSlot slot>
                                                                    <ul class="w-full">{move || rating_breakdown()}</ul>
                                                                </SectionSlot>
                                                            </SectionInnerCard>
                                                        </div>
                                                    </SectionSlot>
                                                </SectionCard>

                                                <SectionCard
                                                    title=format!("Pages Wrapped")
                                                    caption=format!("Here is how many pages you read!")
                                                >
                                                    <SectionSlot slot>
                                                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-2">
                                                            <SectionInnerCard
                                                                title=format!("Total Pages Read")
                                                                value=format!("{} pages", page_section.total_pages.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Longest Book Read")
                                                                value=format!("{} pages", page_section.longest_pages.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Shortest Book Read")
                                                                value=format!("{} pages", page_section.shortest_pages.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Average Book Length")
                                                                value=format!("{:.2} pages", page_section.average_pages)
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Total Books Read")
                                                                value=format!("{} books", page_section.total_books.to_formatted_string(&Locale::en))
                                                            />
                                                        </div>
                                                    </SectionSlot>
                                                </SectionCard>

                                                <SectionCard
                                                    title=format!("Authors Wrapped")
                                                    caption=format!("Here is who you read!")
                                                >
                                                    <SectionSlot slot>
                                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2">
                                                            <SectionInnerCard
                                                                title=format!("Total Authors Read")
                                                                value=format!("{} authors", author_section.total_authors.to_formatted_string(&Locale::en))
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Top 5 Authors")
                                                            >
                                                                <SectionSlot slot>
                                                                    <ul class="w-full">{move || author_breakdown()}</ul>
                                                                </SectionSlot>
                                                            </SectionInnerCard>
                                                        </div>
                                                    </SectionSlot>
                                                </SectionCard>

                                                <SectionCard
                                                    title=format!("Velocity Wrapped")
                                                    caption=format!("Here is how fast you read!")
                                                >
                                                    <SectionSlot slot>
                                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2">
                                                            <SectionInnerCard
                                                                title=format!("Average Speed")
                                                                value=format!("{:.2} days", speed_section.average_speed)
                                                            />
                                                            <SectionInnerCard
                                                                title=format!("Top Months for Reading")
                                                            >
                                                                <SectionSlot slot>
                                                                    <ul class="w-full">{move || months_breakdown()}</ul>
                                                                </SectionSlot>
                                                            </SectionInnerCard>
                                                        </div>
                                                    </SectionSlot>
                                                </SectionCard>
                                            </div>
                                        }.into_any()
                                    }
                                    None => {
                                        view! {
                                            <SectionCard
                                                title=format!("Reading Wrapped")
                                                caption=format!("See a summary of your reading!")
                                            >
                                                <SectionSlot slot>
                                                    <p class="text-amber-700 py-6 font-medium">"We are processing your statistics..."</p>
                                                </SectionSlot>
                                            </SectionCard>
                                        }.into_any()
                                    }
                                }
                            }}
                        </div>
                    </Show>
                </div>
            </Card>
        </div>
    }
}

// Common components

#[slot]
struct SectionSlot {
    children: ChildrenFn
}

#[component]
fn SectionCard(
    title: String,
    caption: String,
    section_slot: SectionSlot
) -> impl IntoView {
    view! {
        <div class="py-3">
            <Card class="bg-white/70 backdrop-blur-sm border border-slate-200/60 shadow-sm rounded-2xl p-4 transition-all hover:shadow-md">
                <CardHeader class="pb-3 border-b border-slate-100 mb-4">
                    <Body1 class="text-lg font-semibold text-slate-900">
                        <b>{format!("{}", title)}</b>
                    </Body1>
                    <CardHeaderDescription slot>
                        <Caption1 class="text-slate-500 text-xs mt-0.5">{format!("{}", caption)}</Caption1>
                    </CardHeaderDescription>
                </CardHeader>
                <div>
                    {move || (section_slot.children)().into_any()}
                </div>
            </Card>
        </div>
    }
}

#[component]
fn SectionInnerCard(
    title: String,
    #[prop(optional)]
    value: Option<String>,
    #[prop(optional)]
    section_slot: Option<SectionSlot>
) -> impl IntoView {
    view! {
        <div class="h-full">
            <Card class="bg-slate-50/80 border border-slate-200/70 rounded-xl p-4 h-full flex flex-col justify-between hover:bg-slate-50 transition-all">
                <CardHeader class="pb-2">
                    <Body1 class="text-xs font-bold uppercase tracking-wider text-slate-500">
                        <b>{format!("{}", title)}</b>
                    </Body1>
                </CardHeader>
                <div class="pt-1 flex-grow flex items-center">
                    {match section_slot {
                        Some(section_slot) => (section_slot.children)().into_any(),
                        None => view! { <p class="text-xl font-bold text-slate-800 w-full text-center">{format!("{}", value.as_deref().unwrap_or("Unknown"))}</p> }.into_any()
                    }}
                </div>
            </Card>
        </div>
    }
}
