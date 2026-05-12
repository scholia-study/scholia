import { useLocation } from "@tanstack/react-router";
import { useEffect, useState } from "react";

export function ScrollToTop() {
    const [visible, setVisible] = useState(false);
    const { pathname } = useLocation();

    useEffect(() => {
        const onScroll = () => setVisible(window.scrollY > 300);
        window.addEventListener("scroll", onScroll, { passive: true });
        return () => window.removeEventListener("scroll", onScroll);
    }, []);

    if (pathname.startsWith("/books")) return null;
    if (!visible) return null;

    return (
        <button
            type="button"
            onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
            className="fixed bottom-6 right-6 z-50 bg-stone-800 text-white rounded-full p-2.5 shadow-lg hover:bg-stone-700 transition-colors"
            title="Scroll to top"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
            >
                <title>Scroll to top</title>
                <path d="m18 15-6-6-6 6" />
            </svg>
        </button>
    );
}
