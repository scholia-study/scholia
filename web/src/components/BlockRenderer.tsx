import parse from "html-react-parser";
import type { ContentBlockResponse, SentenceResponse } from "../api/model";

export function Sentence({
  sentence,
  isSelected,
  onSelect,
}: {
  sentence: SentenceResponse;
  isSelected: boolean;
  onSelect: (sentence: SentenceResponse) => void;
}) {
  return (
    <>
      <span
        onClick={() => onSelect(sentence)}
        className={`cursor-pointer transition-colors rounded-sm ${
          isSelected ? "bg-amber-200" : "hover:bg-stone-200"
        }`}
      >{parse(sentence.html)}</span>{" "}
    </>
  );
}

export function Block({
  block,
  selectedSentenceId,
  onSelectSentence,
}: {
  block: ContentBlockResponse;
  selectedSentenceId: string | null;
  onSelectSentence: (sentence: SentenceResponse) => void;
}) {
  switch (block.block_type) {
    case "heading":
      return (
        <h2 className="text-2xl font-bold mt-8 mb-6 text-stone-900">
          {parse(block.html)}
        </h2>
      );
    case "paragraph":
      return (
        <p className="mb-4 leading-relaxed text-stone-700">
          {block.sentences.map((s) => (
            <Sentence
              key={s.id}
              sentence={s}
              isSelected={s.id === selectedSentenceId}
              onSelect={onSelectSentence}
            />
          ))}
        </p>
      );
    case "footnote":
      return (
        <div className="mb-4 ml-8 text-sm text-stone-500 italic border-l-2 border-stone-200 pl-4">
          {block.sentences.map((s) => (
            <Sentence
              key={s.id}
              sentence={s}
              isSelected={s.id === selectedSentenceId}
              onSelect={onSelectSentence}
            />
          ))}
        </div>
      );
    case "separator":
      return <hr className="my-8 border-stone-200" />;
    default:
      return (
        <div className="mb-4">
          {parse(block.html)}
        </div>
      );
  }
}
