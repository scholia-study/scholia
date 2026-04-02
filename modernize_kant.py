import os
import re

def modernize_text(text):
    # 1. Long s
    text = text.replace('ſ', 's')
    
    # 2. 'th' to 't' in common words (case-insensitive for the word, but preserving case)
    th_words = [
        'Theil', 'Noth', 'Urtheil', 'Vortheil', 'beurtheilen', 'thun', 'nothwendig', 
        'unterthänig', 'Wachsthum', 'Eigenthümliches', 'Eigenthum', 'Rath', 'Thor', 
        'That', 'Vertheidigung', 'benöthigten', 'Mittheilung', 'Vertheidiger',
        'Thier', 'Thür', 'Thon', 'Thräne', 'Thron', 'beurtheilt', 'ungetheilte',
        'getheilt', 'Vortheilhafte', 'nothwendiges', 'thätig', 'Thätigkeit',
        'Nothwendigkeit'
    ]
    for word in th_words:
        pattern = re.compile(re.escape(word), re.IGNORECASE)
        def replace_th(match):
            m = match.group(0)
            return m.replace('Th', 'T').replace('th', 't')
        text = pattern.sub(replace_th, text)

    # 3. 'ey' to 'ei'
    text = re.sub(r'\b([Bb])ey\b', r'\1ei', text)
    text = re.sub(r'\b([Ss])eyn\b', r'\1ein', text)
    text = re.sub(r'\b([Zz])wey\b', r'\1wei', text)
    text = text.replace('zweyerlei', 'zweierlei')
    
    # 4. Loanwords C -> K/Z
    replacements = {
        r'\bObject': 'Objekt',
        r'\bobject': 'objekt',
        r'\bCapitel': 'Kapitel',
        r'\bcapitel': 'kapitel',
        r'\bPrincip': 'Prinzip',
        r'\bprincip': 'prinzip',
        r'\bSpeculation': 'Spekulation',
        r'\bspeculation': 'spekulation',
        r'\bSpeculative': 'Spekulative',
        r'\bspeculative': 'spekulative',
        r'\bSpeculativ': 'Spekulativ',
        r'\bspeculativ': 'spekulativ',
        r'\bPublicum': 'Publikum',
        r'\bpublicum': 'publikum',
        r'\bProduct': 'Produkt',
        r'\bproduct': 'produkt',
        r'\bCausalität': 'Kausalität',
        r'\bcausalität': 'kausalität',
        r'\bConstruction': 'Konstruktion',
        r'\bconstruction': 'konstruktion',
        r'\bScepticism': 'Skeptizismus',
        r'\bscepticism': 'skeptizismus',
        r'\bIdealism': 'Idealismus',
        r'\bidealism': 'idealismus',
        r'\bCopernicus': 'Kopernikus',
        r'\bcopernicus': 'kopernikus',
        r'\bPhysik\b': 'Physik',
        r'\bLogic\b': 'Logik',
        r'transscendental': 'transzendental',
        r'Transscendental': 'Transzendental',
        r'Tractat': 'Traktat',
        r'Subject': 'Subjekt',
        r'subject': 'subjekt',
        r'correspondirend': 'korrespondierend',
        r'Correspond': 'Korrespond',
    }
    for pattern, repl in replacements.items():
        text = re.sub(pattern, repl, text)

    # 5. -niß -> -nis
    text = re.sub(r'niß', 'nis', text)
    
    # 6. Specific names/fixes from Zedlitz dedication and OCR
    text = text.replace('Zedliß', 'Zedlitz')
    text = text.replace('Beschüßers', 'Beschützers')
    
    # 7. daß -> dass
    text = text.replace('daß', 'dass')
    text = text.replace('Daß', 'Dass')
    
    # 8. muss, musste, müssen
    text = text.replace('muß', 'muss')
    text = text.replace('Muß', 'Muss')
    text = text.replace('mußte', 'musste')
    text = text.replace('müßte', 'müsste')
    
    # 9. ß -> tz or z or ss (OCR errors and modernization)
    text = text.replace('Geseße', 'Gesetze')
    text = text.replace('Geseß', 'Gesetz')
    text = text.replace('Nußen', 'Nutzen')
    text = text.replace('Nuß', 'Nutz')
    text = text.replace('Gegensaße', 'Gegensatze')
    text = text.replace('zulezt', 'zuletzt')
    text = text.replace('besizen', 'besitzen')
    text = text.replace('beſeßen', 'besessen')
    text = text.replace('Saz', 'Satz')
    text = text.replace('Sak', 'Satz')
    text = text.replace('Säße', 'Sätze')
    text = text.replace('Plaz', 'Platz')
    text = text.replace('Gegensaze', 'Gegensatze')
    text = text.replace('beſeßen', 'besessen')
    text = text.replace('wißen', 'wissen')
    text = text.replace('Wiße', 'Wissen')
    text = text.replace('wißbegierige', 'wissbegierige')
    text = text.replace('Wißbegierde', 'Wissbegierde')

    # 10. OCR fixes (observed in text)
    text = text.replace('Gaug', 'Gang')
    text = text.replace('Erperiment', 'Experiment')
    text = text.replace('Mernunft', 'Vernunft')
    text = text.replace('Umånderung', 'Umänderung')
    text = text.replace('gångeln', 'gängeln')
    text = text.replace('ſpåterer', 'späterer')
    text = text.replace('Denfart', 'Denkart')
    text = text.replace('widerfinnische', 'widersinnische')
    text = text.replace('insgeſammt', 'insgesamt')
    text = text.replace('insgesammt', 'insgesamt')
    text = text.replace('übele', 'üble')
    text = text.replace('Wirthschaft', 'Wirtschaft')
    text = text.replace('literärischen', 'literarischen')
    text = text.replace('überschwenglicher', 'überschwänglicher')
    text = text.replace('u. f. w.', 'u. s. w.')
    text = text.replace('denu', 'denn')
    text = text.replace('eristire', 'existiere')
    text = text.replace('Eristenz', 'Existenz')
    text = text.replace('Idealisut', 'Idealismus')
    text = text.replace('Idealismuss', 'Idealismus')
    text = text.replace('Period ', 'Periode ')
    text = text.replace('Period.', 'Periode.')
    text = text.replace('intellectuell', 'intellektuell')
    text = text.replace('correspond', 'korrespond')
    text = text.replace('Correspond', 'Korrespond')
    text = text.replace('verabſäumen', 'versäumen')
    text = text.replace('allervörderst', 'allervorderst')

    return text

source_dir = 'assets/kant1_md_reviewed'
target_dir = 'assets/kant1_md_modernized'

if not os.path.exists(target_dir):
    os.makedirs(target_dir)

for filename in os.listdir(source_dir):
    if filename.endswith('.md'):
        with open(os.path.join(source_dir, filename), 'r', encoding='utf-8') as f:
            content = f.read()
        
        modernized_content = modernize_text(content)
        
        with open(os.path.join(target_dir, filename), 'w', encoding='utf-8') as f:
            f.write(modernized_content)
        print(f"Processed {filename}")