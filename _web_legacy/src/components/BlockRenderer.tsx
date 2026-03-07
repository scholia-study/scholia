import parse from "html-react-parser";
import type { ContentBlockResponse, SentenceResponse, PageMarkerResponse } from "../api/model";

export interface MarginSettings {
  enabledSystems: Set<string>
  systemSides: Record<string, 'left' | 'right'>
}

function MarginNotes({
  markers,
  side,
}: {
  markers: PageMarkerResponse[]
  side: 'left' | 'right'
}) {
  return (
    <span
      className={`absolute flex gap-1 whitespace-nowrap text-[10px] text-stone-400 select-none ${
        side === 'left'
          ? 'right-full mr-2 justify-end'
          : 'left-full ml-2 justify-start'
      }`}
      style={{ lineHeight: 'inherit' }}
    >
      {markers.map((pm, i) => (
        <span key={`${pm.system_slug}-${pm.ref_value}-${i}`} title={`${pm.system_slug}: ${pm.ref_value}`}>
          {pm.ref_value}
        </span>
      ))}
    </span>
  )
}

export function Sentence({
  sentence,
  isSelected,
  onSelect,
  marginSettings,
}: {
  sentence: SentenceResponse;
  isSelected: boolean;
  onSelect: (sentence: SentenceResponse) => void;
  marginSettings?: MarginSettings;
}) {
  let leftMarkers: PageMarkerResponse[] | undefined
  let rightMarkers: PageMarkerResponse[] | undefined

  if (marginSettings && marginSettings.enabledSystems.size > 0 && sentence.page_markers.length > 0) {
    for (const pm of sentence.page_markers) {
      if (!marginSettings.enabledSystems.has(pm.system_slug)) continue
      const side = marginSettings.systemSides[pm.system_slug] ?? 'right'
      if (side === 'left') {
        (leftMarkers ??= []).push(pm)
      } else {
        (rightMarkers ??= []).push(pm)
      }
    }
  }

  return (
    <>
      {leftMarkers && <MarginNotes markers={leftMarkers} side="left" />}
      {rightMarkers && <MarginNotes markers={rightMarkers} side="right" />}
      <span
        onClick={() => onSelect(sentence)}
        className={`cursor-pointer transition-colors rounded-sm ${
          isSelected ? "bg-amber-200" : "hover:bg-stone-200"
        }`}
      >{parse(sentence.html)}</span>{" "}
    </>
  );
}

function HeadingSentence({
  sentence,
  marginSettings,
}: {
  sentence: SentenceResponse
  marginSettings?: MarginSettings
}) {
  let leftMarkers: PageMarkerResponse[] | undefined
  let rightMarkers: PageMarkerResponse[] | undefined

  if (marginSettings && marginSettings.enabledSystems.size > 0 && sentence.page_markers.length > 0) {
    for (const pm of sentence.page_markers) {
      if (!marginSettings.enabledSystems.has(pm.system_slug)) continue
      const side = marginSettings.systemSides[pm.system_slug] ?? 'right'
      if (side === 'left') {
        (leftMarkers ??= []).push(pm)
      } else {
        (rightMarkers ??= []).push(pm)
      }
    }
  }

  return (
    <>
      {leftMarkers && <MarginNotes markers={leftMarkers} side="left" />}
      {rightMarkers && <MarginNotes markers={rightMarkers} side="right" />}
      <span>{parse(sentence.html)}</span>{" "}
    </>
  )
}

export function Block({
  block,
  selectedSentenceId,
  onSelectSentence,
  marginSettings,
}: {
  block: ContentBlockResponse;
  selectedSentenceId: string | null;
  onSelectSentence: (sentence: SentenceResponse) => void;
  marginSettings?: MarginSettings;
}) {
  switch (block.block_type) {
    case "heading":
      return (
        <h2 className="relative text-2xl font-bold mt-8 mb-6 text-stone-900">
          {block.sentences.length > 0
            ? block.sentences.map((s) => (
                <HeadingSentence
                  key={s.id}
                  sentence={s}
                  marginSettings={marginSettings}
                />
              ))
            : parse(block.html)}
        </h2>
      );
    case "paragraph":
      return (
        <p className="relative mb-4 leading-relaxed text-stone-700">
          {block.sentences.map((s) => (
            <Sentence
              key={s.id}
              sentence={s}
              isSelected={s.id === selectedSentenceId}
              onSelect={onSelectSentence}
              marginSettings={marginSettings}
            />
          ))}
        </p>
      );
    case "footnote":
      return (
        <div className="relative mb-4 ml-8 text-sm text-stone-500 italic border-l-2 border-stone-200 pl-4">
          {block.sentences.map((s) => (
            <Sentence
              key={s.id}
              sentence={s}
              isSelected={s.id === selectedSentenceId}
              onSelect={onSelectSentence}
              marginSettings={marginSettings}
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
