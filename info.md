from google.cloud import documentai_v1 as documentai

def get_structured_reader_data(project_id, location, processor_id, file_path):
client = documentai.DocumentProcessorServiceClient()
name = client.processor_path(project_id, location, processor_id)

    # Read the file into memory
    with open(file_path, "rb") as image:
        image_content = image.read()

    raw_document = documentai.RawDocument(content=image_content, mime_type="image/jpeg")
    request = documentai.ProcessRequest(name=name, raw_document=raw_document)

    result = client.process_document(request=request)
    document = result.document

    # Helper to grab text via offsets
    def get_text(el):
        return "".join([document.text[int(segment.start_index):int(segment.end_index)]
                        for segment in el.layout.text_anchor.text_segments])

    hierarchy = []
    current_section = {"heading": "Front Matter", "content": []}

    # Document AI Layout Parser identifies 'entities' as structural blocks
    for entity in document.entities:
        type_ = entity.type_
        text_content = get_text(entity)

        if "heading" in type_.lower():
            # If we hit a new heading, save the old section and start a new one
            hierarchy.append(current_section)
            current_section = {"heading": text_content.strip(), "content": []}
        else:
            current_section["content"].append(text_content.strip())

    hierarchy.append(current_section) # Add the last section
    return hierarchy

# Example usage:

# data = get_structured_reader_data("my-project", "us", "my-processor-id", "kant_page_1.jpg")
