import { useListBooks } from "../api/books/books";

interface BookPickerPanelProps {
    onPickBook: (bookSlug: string) => void;
    onClose: (() => void) | undefined;
}

export function BookPickerPanel({ onPickBook, onClose }: BookPickerPanelProps) {
    const { data, isLoading, error } = useListBooks();
    const books = data?.data;

    return (
        <div className="flex w-64 shrink-0 border-r border-stone-200">
            <div className="flex-1 flex flex-col min-w-0">
                <div className="flex items-center gap-2 px-3 py-2 border-b border-stone-200 bg-white shrink-0">
                    <span className="text-sm text-stone-500 flex-1">
                        Select a book
                    </span>
                    {onClose && (
                        <button
                            onClick={onClose}
                            className="text-stone-400 hover:text-stone-600 text-lg leading-none"
                            title="Close panel"
                        >
                            &times;
                        </button>
                    )}
                </div>
                <div className="flex-1 overflow-y-auto p-2">
                    {isLoading && (
                        <p className="text-stone-400 text-sm p-2">Loading...</p>
                    )}
                    {error ? (
                        <p className="text-red-500 text-sm p-2">
                            Failed to load books.
                        </p>
                    ) : null}
                    {books && (
                        <ul className="space-y-1">
                            {books.map((book) => (
                                <li key={book.id}>
                                    <button
                                        onClick={() => onPickBook(book.slug)}
                                        className="block w-full text-left px-2 py-1.5 rounded hover:bg-stone-100 transition-colors"
                                    >
                                        <div className="text-sm text-stone-900">
                                            {book.title}
                                        </div>
                                        <div className="text-xs text-stone-500">
                                            {book.author}
                                        </div>
                                    </button>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
            </div>
        </div>
    );
}
