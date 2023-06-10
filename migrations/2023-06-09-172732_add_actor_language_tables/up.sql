create table language(
    id serial primary key,
    code text,
    name text
);

create table local_user_language(
    id serial primary key,
    local_user_id int references local_user on update cascade on delete cascade,
    language_id int references language on update cascade on delete cascade,
    unique(local_user_id, language_id)
);

alter table local_user rename column lang to interface_language;

insert into language(id, code, name) values (0, 'und', 'Undetermined');
alter table posts add column language_id int references language not null default 0;

-- insert the language codes
insert into language(code, name) values ('aa', 'Afaraf');
insert into language(code, name) values ('ab', 'аҧсуа бызшәа');
insert into language(code, name) values ('ae', 'avesta');
insert into language(code, name) values ('af', 'Afrikaans');
insert into language(code, name) values ('ak', 'Akan');
insert into language(code, name) values ('am', 'አማርኛ');
insert into language(code, name) values ('an', 'aragonés');
insert into language(code, name) values ('ar', 'اَلْعَرَبِيَّةُ');
insert into language(code, name) values ('as', 'অসমীয়া');
insert into language(code, name) values ('av', 'авар мацӀ');
insert into language(code, name) values ('ay', 'aymar aru');
insert into language(code, name) values ('az', 'azərbaycan dili');
insert into language(code, name) values ('ba', 'башҡорт теле');
insert into language(code, name) values ('be', 'беларуская мова');
insert into language(code, name) values ('bg', 'български език');
insert into language(code, name) values ('bi', 'Bislama');
insert into language(code, name) values ('bm', 'bamanankan');
insert into language(code, name) values ('bn', 'বাংলা');
insert into language(code, name) values ('bo', 'བོད་ཡིག');
insert into language(code, name) values ('br', 'brezhoneg');
insert into language(code, name) values ('bs', 'bosanski jezik');
insert into language(code, name) values ('ca', 'Català');
insert into language(code, name) values ('ce', 'нохчийн мотт');
insert into language(code, name) values ('ch', 'Chamoru');
insert into language(code, name) values ('co', 'corsu');
insert into language(code, name) values ('cr', 'ᓀᐦᐃᔭᐍᐏᐣ');
insert into language(code, name) values ('cs', 'čeština');
insert into language(code, name) values ('cu', 'ѩзыкъ словѣньскъ');
insert into language(code, name) values ('cv', 'чӑваш чӗлхи');
insert into language(code, name) values ('cy', 'Cymraeg');
insert into language(code, name) values ('da', 'dansk');
insert into language(code, name) values ('de', 'Deutsch');
insert into language(code, name) values ('dv', 'ދިވެހި');
insert into language(code, name) values ('dz', 'རྫོང་ཁ');
insert into language(code, name) values ('ee', 'Eʋegbe');
insert into language(code, name) values ('el', 'Ελληνικά');
insert into language(code, name) values ('en', 'English');
insert into language(code, name) values ('eo', 'Esperanto');
insert into language(code, name) values ('es', 'Español');
insert into language(code, name) values ('et', 'eesti');
insert into language(code, name) values ('eu', 'euskara');
insert into language(code, name) values ('fa', 'فارسی');
insert into language(code, name) values ('ff', 'Fulfulde');
insert into language(code, name) values ('fi', 'suomi');
insert into language(code, name) values ('fj', 'vosa Vakaviti');
insert into language(code, name) values ('fo', 'føroyskt');
insert into language(code, name) values ('fr', 'Français');
insert into language(code, name) values ('fy', 'Frysk');
insert into language(code, name) values ('ga', 'Gaeilge');
insert into language(code, name) values ('gd', 'Gàidhlig');
insert into language(code, name) values ('gl', 'galego');
insert into language(code, name) values ('gn', E'Avañe\'ẽ');
insert into language(code, name) values ('gu', 'ગુજરાતી');
insert into language(code, name) values ('gv', 'Gaelg');
insert into language(code, name) values ('ha', 'هَوُسَ');
insert into language(code, name) values ('he', 'עברית');
insert into language(code, name) values ('hi', 'हिन्दी');
insert into language(code, name) values ('ho', 'Hiri Motu');
insert into language(code, name) values ('hr', 'Hrvatski');
insert into language(code, name) values ('ht', 'Kreyòl ayisyen');
insert into language(code, name) values ('hu', 'magyar');
insert into language(code, name) values ('hy', 'Հայերեն');
insert into language(code, name) values ('hz', 'Otjiherero');
insert into language(code, name) values ('ia', 'Interlingua');
insert into language(code, name) values ('id', 'Bahasa Indonesia');
insert into language(code, name) values ('ie', 'Interlingue');
insert into language(code, name) values ('ig', 'Asụsụ Igbo');
insert into language(code, name) values ('ii', 'ꆈꌠ꒿ Nuosuhxop');
insert into language(code, name) values ('ik', 'Iñupiaq');
insert into language(code, name) values ('io', 'Ido');
insert into language(code, name) values ('is', 'Íslenska');
insert into language(code, name) values ('it', 'Italiano');
insert into language(code, name) values ('iu', 'ᐃᓄᒃᑎᑐᑦ');
insert into language(code, name) values ('ja', '日本語');
insert into language(code, name) values ('jv', 'basa Jawa');
insert into language(code, name) values ('ka', 'ქართული');
insert into language(code, name) values ('kg', 'Kikongo');
insert into language(code, name) values ('ki', 'Gĩkũyũ');
insert into language(code, name) values ('kj', 'Kuanyama');
insert into language(code, name) values ('kk', 'қазақ тілі');
insert into language(code, name) values ('kl', 'kalaallisut');
insert into language(code, name) values ('km', 'ខេមរភាសា');
insert into language(code, name) values ('kn', 'ಕನ್ನಡ');
insert into language(code, name) values ('ko', '한국어');
insert into language(code, name) values ('kr', 'Kanuri');
insert into language(code, name) values ('ks', 'कश्मीरी');
insert into language(code, name) values ('ku', 'Kurdî');
insert into language(code, name) values ('kv', 'коми кыв');
insert into language(code, name) values ('kw', 'Kernewek');
insert into language(code, name) values ('ky', 'Кыргызча');
insert into language(code, name) values ('la', 'latine');
insert into language(code, name) values ('lb', 'Lëtzebuergesch');
insert into language(code, name) values ('lg', 'Luganda');
insert into language(code, name) values ('li', 'Limburgs');
insert into language(code, name) values ('ln', 'Lingála');
insert into language(code, name) values ('lo', 'ພາສາລາວ');
insert into language(code, name) values ('lt', 'lietuvių kalba');
insert into language(code, name) values ('lu', 'Kiluba');
insert into language(code, name) values ('lv', 'latviešu valoda');
insert into language(code, name) values ('mg', 'fiteny malagasy');
insert into language(code, name) values ('mh', 'Kajin M̧ajeļ');
insert into language(code, name) values ('mi', 'te reo Māori');
insert into language(code, name) values ('mk', 'македонски јазик');
insert into language(code, name) values ('ml', 'മലയാളം');
insert into language(code, name) values ('mn', 'Монгол хэл');
insert into language(code, name) values ('mr', 'मराठी');
insert into language(code, name) values ('ms', 'Bahasa Melayu');
insert into language(code, name) values ('mt', 'Malti');
insert into language(code, name) values ('my', 'ဗမာစာ');
insert into language(code, name) values ('na', 'Dorerin Naoero');
insert into language(code, name) values ('nb', 'Norsk bokmål');
insert into language(code, name) values ('nd', 'isiNdebele');
insert into language(code, name) values ('ne', 'नेपाली');
insert into language(code, name) values ('ng', 'Owambo');
insert into language(code, name) values ('nl', 'Nederlands');
insert into language(code, name) values ('nn', 'Norsk nynorsk');
insert into language(code, name) values ('no', 'Norsk');
insert into language(code, name) values ('nr', 'isiNdebele');
insert into language(code, name) values ('nv', 'Diné bizaad');
insert into language(code, name) values ('ny', 'chiCheŵa');
insert into language(code, name) values ('oc', 'occitan');
insert into language(code, name) values ('oj', 'ᐊᓂᔑᓈᐯᒧᐎᓐ');
insert into language(code, name) values ('om', 'Afaan Oromoo');
insert into language(code, name) values ('or', 'ଓଡ଼ିଆ');
insert into language(code, name) values ('os', 'ирон æвзаг');
insert into language(code, name) values ('pa', 'ਪੰਜਾਬੀ');
insert into language(code, name) values ('pi', 'पाऴि');
insert into language(code, name) values ('pl', 'Polski');
insert into language(code, name) values ('ps', 'پښتو');
insert into language(code, name) values ('pt', 'Português');
insert into language(code, name) values ('qu', 'Runa Simi');
insert into language(code, name) values ('rm', 'rumantsch grischun');
insert into language(code, name) values ('rn', 'Ikirundi');
insert into language(code, name) values ('ro', 'Română');
insert into language(code, name) values ('ru', 'Русский');
insert into language(code, name) values ('rw', 'Ikinyarwanda');
insert into language(code, name) values ('sa', 'संस्कृतम्');
insert into language(code, name) values ('sc', 'sardu');
insert into language(code, name) values ('sd', 'सिन्धी');
insert into language(code, name) values ('se', 'Davvisámegiella');
insert into language(code, name) values ('sg', 'yângâ tî sängö');
insert into language(code, name) values ('si', 'සිංහල');
insert into language(code, name) values ('sk', 'slovenčina');
insert into language(code, name) values ('sl', 'slovenščina');
insert into language(code, name) values ('sm', E'gagana fa\'a Samoa');
insert into language(code, name) values ('sn', 'chiShona');
insert into language(code, name) values ('so', 'Soomaaliga');
insert into language(code, name) values ('sq', 'Shqip');
insert into language(code, name) values ('sr', 'српски језик');
insert into language(code, name) values ('ss', 'SiSwati');
insert into language(code, name) values ('st', 'Sesotho');
insert into language(code, name) values ('su', 'Basa Sunda');
insert into language(code, name) values ('sv', 'Svenska');
insert into language(code, name) values ('sw', 'Kiswahili');
insert into language(code, name) values ('ta', 'தமிழ்');
insert into language(code, name) values ('te', 'తెలుగు');
insert into language(code, name) values ('tg', 'тоҷикӣ');
insert into language(code, name) values ('th', 'ไทย');
insert into language(code, name) values ('ti', 'ትግርኛ');
insert into language(code, name) values ('tk', 'Türkmençe');
insert into language(code, name) values ('tl', 'Wikang Tagalog');
insert into language(code, name) values ('tn', 'Setswana');
insert into language(code, name) values ('to', 'faka Tonga');
insert into language(code, name) values ('tr', 'Türkçe');
insert into language(code, name) values ('ts', 'Xitsonga');
insert into language(code, name) values ('tt', 'татар теле');
insert into language(code, name) values ('tw', 'Twi');
insert into language(code, name) values ('ty', 'Reo Tahiti');
insert into language(code, name) values ('ug', 'ئۇيغۇرچە‎');
insert into language(code, name) values ('uk', 'Українська');
insert into language(code, name) values ('ur', 'اردو');
insert into language(code, name) values ('uz', 'Ўзбек');
insert into language(code, name) values ('ve', 'Tshivenḓa');
insert into language(code, name) values ('vi', 'Tiếng Việt');
insert into language(code, name) values ('vo', 'Volapük');
insert into language(code, name) values ('wa', 'walon');
insert into language(code, name) values ('wo', 'Wollof');
insert into language(code, name) values ('xh', 'isiXhosa');
insert into language(code, name) values ('yi', 'ייִדיש');
insert into language(code, name) values ('yo', 'Yorùbá');
insert into language(code, name) values ('za', 'Saɯ cueŋƅ');
insert into language(code, name) values ('zh', '中文');
insert into language(code, name) values ('zu', 'isiZulu');