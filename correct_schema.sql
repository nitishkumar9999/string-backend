--
-- PostgreSQL database dump
--

\restrict MLHnqPH6LGZ2geRf5l1OQfgxYHz9bEaFruayJZHpKvzXgOKgTwOrPfQrLZJJzAa

-- Dumped from database version 18.1 (Ubuntu 18.1-1.pgdg24.04+2)
-- Dumped by pg_dump version 18.1 (Ubuntu 18.1-1.pgdg24.04+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: public; Type: SCHEMA; Schema: -; Owner: nitish
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO nitish;

--
-- Name: cleanup_expired_csrf_tokens(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.cleanup_expired_csrf_tokens() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM csrf_tokens WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$;


ALTER FUNCTION public.cleanup_expired_csrf_tokens() OWNER TO postgres;

--
-- Name: cleanup_expired_sessions(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.cleanup_expired_sessions() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_deleted_count INTEGER;
BEGIN
    DELETE FROM sessions
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    RETURN v_deleted_count;
END;
$$;


ALTER FUNCTION public.cleanup_expired_sessions() OWNER TO postgres;

--
-- Name: cleanup_old_rate_limits(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.cleanup_old_rate_limits() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM rate_limits
    WHERE window_start < NOW() - INTERVAL '24 hours';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$;


ALTER FUNCTION public.cleanup_old_rate_limits() OWNER TO nitish;

--
-- Name: decrement_answer_comment_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_answer_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        IF NEW.answer_id IS NOT NULL THEN
            UPDATE answers SET comment_count = GREATEST(comment_count - 1, 0)
            WHERE id = NEW.answer_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.decrement_answer_comment_count() OWNER TO postgres;

--
-- Name: decrement_comment_helpful_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_comment_helpful_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE comments SET helpful_count = GREATEST(helpful_count - 1, 0)
    WHERE id = OLD.comment_id;
    RETURN OLD;
END;
$$;


ALTER FUNCTION public.decrement_comment_helpful_count() OWNER TO postgres;

--
-- Name: decrement_post_comment_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_post_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        IF NEW.post_id IS NOT NULL THEN
            UPDATE posts SET comment_count = GREATEST(comment_count - 1, 0)
            WHERE id = NEW.post_id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.decrement_post_comment_count() OWNER TO postgres;

--
-- Name: decrement_post_refract_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_post_refract_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE posts SET refract_count = GREATEST(refract_count - 1, 0)
        WHERE id = NEW.original_post_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.decrement_post_refract_count() OWNER TO postgres;

--
-- Name: decrement_question_answer_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_question_answer_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE questions SET answer_count = GREATEST(answer_count - 1, 0)
        WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.decrement_question_answer_count() OWNER TO postgres;

--
-- Name: decrement_tag_usage_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.decrement_tag_usage_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE tags SET usage_count = GREATEST(usage_count - 1, 0)
    WHERE id = OLD.tag_id;
    RETURN OLD;
END;
$$;


ALTER FUNCTION public.decrement_tag_usage_count() OWNER TO postgres;

--
-- Name: increment_answer_comment_count(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.increment_answer_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.parent_comment_id IS NULL THEN
        UPDATE answers SET comment_count = comment_count + 1 WHERE id = NEW.answer_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_answer_comment_count() OWNER TO nitish;

--
-- Name: increment_answer_echo_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_answer_echo_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.answer_id IS NOT NULL THEN
        UPDATE answers SET echo_count = echo_count + 1 
        WHERE id = NEW.answer_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_answer_echo_count() OWNER TO postgres;

--
-- Name: increment_comment_helpful_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_comment_helpful_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE comments SET helpful_count = helpful_count + 1 
    WHERE id = NEW.comment_id;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_comment_helpful_count() OWNER TO postgres;

--
-- Name: increment_comment_reply_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_comment_reply_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.parent_comment_id IS NOT NULL THEN
        UPDATE comments SET reply_count = reply_count + 1 
        WHERE id = NEW.parent_comment_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_comment_reply_count() OWNER TO postgres;

--
-- Name: increment_post_comment_count(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.increment_post_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.parent_comment_id IS NULL THEN
        UPDATE posts SET comment_count = comment_count + 1 WHERE id = NEW.post_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_post_comment_count() OWNER TO nitish;

--
-- Name: increment_post_echo_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_post_echo_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.post_id IS NOT NULL THEN
        UPDATE posts SET echo_count = echo_count + 1 
        WHERE id = NEW.post_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_post_echo_count() OWNER TO postgres;

--
-- Name: increment_post_refract_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_post_refract_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE posts SET refract_count = refract_count + 1 
    WHERE id = NEW.original_post_id;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_post_refract_count() OWNER TO postgres;

--
-- Name: increment_question_answer_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_question_answer_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE questions SET answer_count = answer_count + 1 
    WHERE id = NEW.question_id;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_question_answer_count() OWNER TO postgres;

--
-- Name: increment_question_comment_count(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.increment_question_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.parent_comment_id IS NULL THEN
        UPDATE questions SET comment_count = comment_count + 1 WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_question_comment_count() OWNER TO nitish;

--
-- Name: increment_question_echo_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_question_echo_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.question_id IS NOT NULL THEN
        UPDATE questions SET echo_count = echo_count + 1 
        WHERE id = NEW.question_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_question_echo_count() OWNER TO postgres;

--
-- Name: increment_refract_echo_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_refract_echo_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.refract_id IS NOT NULL THEN
        UPDATE refracts SET echo_count = echo_count + 1 
        WHERE id = NEW.refract_id;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_refract_echo_count() OWNER TO postgres;

--
-- Name: increment_tag_usage_count(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.increment_tag_usage_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE tags SET usage_count = usage_count + 1 
    WHERE id = NEW.tag_id;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.increment_tag_usage_count() OWNER TO postgres;

--
-- Name: populate_search_vectors(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.populate_search_vectors() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE posts 
    SET search_vector = 
        setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(content_raw, '')), 'B')
    WHERE search_vector IS NULL;

    UPDATE questions 
    SET search_vector = 
        setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(content_raw, '')), 'B')
    WHERE search_vector IS NULL;

    UPDATE answers 
    SET search_vector = to_tsvector('english', COALESCE(content_raw, ''))
    WHERE search_vector IS NULL;
END;
$$;


ALTER FUNCTION public.populate_search_vectors() OWNER TO nitish;

--
-- Name: rotate_session(uuid, integer); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.rotate_session(p_old_session_id uuid, p_user_id integer) RETURNS uuid
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_new_session_id UUID;
BEGIN
    -- Create new session
    INSERT INTO sessions (user_id, ip_address, user_agent, expires_at, rotated_from)
    SELECT user_id, ip_address, user_agent, 
           NOW() + INTERVAL '30 days', 
           id
    FROM sessions
    WHERE id = p_old_session_id AND user_id = p_user_id
    RETURNING id INTO v_new_session_id;
    
    -- Delete old session
    DELETE FROM sessions WHERE id = p_old_session_id;
    
    RETURN v_new_session_id;
END;
$$;


ALTER FUNCTION public.rotate_session(p_old_session_id uuid, p_user_id integer) OWNER TO postgres;

--
-- Name: search_all(text, integer[], text[], text, integer, real, integer, integer, timestamp with time zone, integer); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.search_all(search_text text DEFAULT NULL::text, tag_ids integer[] DEFAULT NULL::integer[], content_types text[] DEFAULT ARRAY['posts'::text, 'questions'::text], search_mode text DEFAULT 'all'::text, limit_count integer DEFAULT 20, cursor_rank real DEFAULT NULL::real, cursor_matched_tags integer DEFAULT NULL::integer, cursor_engagement integer DEFAULT NULL::integer, cursor_created_at timestamp with time zone DEFAULT NULL::timestamp with time zone, cursor_id integer DEFAULT NULL::integer) RETURNS TABLE(content_type text, id integer, user_id integer, username character varying, avatar_url text, title character varying, content_raw text, content_rendered_html text, created_at timestamp with time zone, echo_count integer, refract_count integer, comment_count integer, answer_count integer, engagement_count integer, rank real, matched_tags integer, slug character varying, is_spam boolean, tag_names text[])
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    WITH posts_results AS (
        SELECT 
            'post'::TEXT AS content_type,
            p.id,
            p.user_id,
            u.username,  
            u.avatar_url,              -- ✅ ADDED
            p.title,
            p.content_raw,
            p.content_rendered_html,
            p.created_at,
            p.echo_count,
            p.refract_count,
            p.comment_count,
            0 AS answer_count,         -- ✅ ADDED (posts don't have answers)
            (p.echo_count + p.refract_count + p.comment_count) AS engagement_count,
            CASE 
                WHEN search_text IS NOT NULL THEN ts_rank(p.search_vector, websearch_to_tsquery('english', search_text))
                ELSE 0
            END AS rank,
            CASE 
                WHEN tag_ids IS NULL OR ARRAY_LENGTH(tag_ids, 1) IS NULL THEN 0
                ELSE (SELECT COUNT(DISTINCT pt.tag_id)::INTEGER
                      FROM post_tags pt
                      WHERE pt.post_id = p.id AND pt.tag_id = ANY(tag_ids))
            END AS matched_tags,
            p.slug,
            p.is_spam,
            COALESCE(                  -- ✅ ADDED
                ARRAY(
                    SELECT t.name::TEXT 
                    FROM post_tags pt 
                    JOIN tags t ON pt.tag_id = t.id 
                    WHERE pt.post_id = p.id
                    ORDER BY t.name
                ), 
                ARRAY[]::TEXT[]
            ) AS tag_names
        FROM posts p
        JOIN users u ON p.user_id = u.id  -- ✅ ADDED JOIN
        WHERE 
            p.deleted_at IS NULL
            AND p.is_spam = FALSE
            AND 'posts' = ANY(content_types)
            AND (search_text IS NULL OR p.search_vector @@ websearch_to_tsquery('english', search_text))
            AND (
                tag_ids IS NULL 
                OR ARRAY_LENGTH(tag_ids, 1) IS NULL
                OR ARRAY_LENGTH(tag_ids, 1) = 0
                OR (
                    CASE WHEN search_mode = 'precise' THEN
                        p.id IN (
                            SELECT pt.post_id 
                            FROM post_tags pt 
                            WHERE pt.tag_id = ANY(tag_ids)
                            GROUP BY pt.post_id
                            HAVING COUNT(DISTINCT pt.tag_id) = ARRAY_LENGTH(tag_ids, 1)
                        )
                    ELSE
                        EXISTS (
                            SELECT 1 FROM post_tags pt 
                            WHERE pt.post_id = p.id AND pt.tag_id = ANY(tag_ids)
                        )
                    END
                )
            )
    ),
    questions_results AS (
        SELECT 
            'question'::TEXT AS content_type,
            q.id,
            q.user_id,
            u.username, 
            u.avatar_url,               -- ✅ ADDED
            q.title,
            q.content_raw,
            q.content_rendered_html,
            q.created_at,
            q.echo_count,
            0 AS refract_count,
            q.comment_count,
            q.answer_count,            -- ✅ ADDED
            (q.echo_count + q.answer_count + q.comment_count) AS engagement_count,
            CASE 
                WHEN search_text IS NOT NULL THEN ts_rank(q.search_vector, websearch_to_tsquery('english', search_text))
                ELSE 0
            END AS rank,
            CASE 
                WHEN tag_ids IS NULL OR ARRAY_LENGTH(tag_ids, 1) IS NULL THEN 0
                ELSE (SELECT COUNT(DISTINCT qt.tag_id)::INTEGER
                      FROM question_tags qt
                      WHERE qt.question_id = q.id AND qt.tag_id = ANY(tag_ids))
            END AS matched_tags,
            q.slug,
            q.is_spam,
            COALESCE(                  -- ✅ ADDED
                ARRAY(
                    SELECT t.name::TEXT 
                    FROM question_tags qt 
                    JOIN tags t ON qt.tag_id = t.id 
                    WHERE qt.question_id = q.id
                    ORDER BY t.name
                ), 
                ARRAY[]::TEXT[]
            ) AS tag_names
        FROM questions q
        JOIN users u ON q.user_id = u.id  -- ✅ ADDED JOIN
        WHERE 
            q.deleted_at IS NULL
            AND q.is_spam = FALSE
            AND 'questions' = ANY(content_types)
            AND (search_text IS NULL OR q.search_vector @@ websearch_to_tsquery('english', search_text))
            AND (
                tag_ids IS NULL 
                OR ARRAY_LENGTH(tag_ids, 1) IS NULL
                OR ARRAY_LENGTH(tag_ids, 1) = 0
                OR (
                    CASE WHEN search_mode = 'precise' THEN
                        q.id IN (
                            SELECT qt.question_id 
                            FROM question_tags qt 
                            WHERE qt.tag_id = ANY(tag_ids)
                            GROUP BY qt.question_id
                            HAVING COUNT(DISTINCT qt.tag_id) = ARRAY_LENGTH(tag_ids, 1)
                        )
                    ELSE
                        EXISTS (
                            SELECT 1 FROM question_tags qt 
                            WHERE qt.question_id = q.id AND qt.tag_id = ANY(tag_ids)
                        )
                    END
                )
            )
    ),
    combined AS (
        SELECT * FROM posts_results
        UNION ALL
        SELECT * FROM questions_results
    )
    SELECT *
    FROM combined
    WHERE 
        cursor_id IS NULL 
        OR (
            CASE WHEN search_text IS NOT NULL THEN
                (combined.rank < cursor_rank) OR 
                (combined.rank = cursor_rank AND combined.matched_tags < cursor_matched_tags) OR
                (combined.rank = cursor_rank AND combined.matched_tags = cursor_matched_tags AND combined.engagement_count < cursor_engagement) OR
                (combined.rank = cursor_rank AND combined.matched_tags = cursor_matched_tags AND combined.engagement_count = cursor_engagement AND combined.created_at < cursor_created_at) OR
                (combined.rank = cursor_rank AND combined.matched_tags = cursor_matched_tags AND combined.engagement_count = cursor_engagement AND combined.created_at = cursor_created_at AND combined.id < cursor_id)
            ELSE
                (combined.matched_tags < cursor_matched_tags) OR
                (combined.matched_tags = cursor_matched_tags AND combined.engagement_count < cursor_engagement) OR
                (combined.matched_tags = cursor_matched_tags AND combined.engagement_count = cursor_engagement AND combined.created_at < cursor_created_at) OR
                (combined.matched_tags = cursor_matched_tags AND combined.engagement_count = cursor_engagement AND combined.created_at = cursor_created_at AND combined.id < cursor_id)
            END
        )
    ORDER BY 
        CASE WHEN search_text IS NOT NULL THEN combined.rank ELSE 0 END DESC,
        combined.matched_tags DESC,
        combined.engagement_count DESC,
        combined.created_at DESC,
        combined.id DESC
    LIMIT limit_count;
END;
$$;


ALTER FUNCTION public.search_all(search_text text, tag_ids integer[], content_types text[], search_mode text, limit_count integer, cursor_rank real, cursor_matched_tags integer, cursor_engagement integer, cursor_created_at timestamp with time zone, cursor_id integer) OWNER TO nitish;

--
-- Name: update_answer_search_vector(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.update_answer_search_vector() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', COALESCE(NEW.content_raw, ''));
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_answer_search_vector() OWNER TO nitish;

--
-- Name: update_post_search_vector(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.update_post_search_vector() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.content_raw, '')), 'B');
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_post_search_vector() OWNER TO nitish;

--
-- Name: update_question_search_vector(); Type: FUNCTION; Schema: public; Owner: nitish
--

CREATE FUNCTION public.update_question_search_vector() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.content_raw, '')), 'B');
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_question_search_vector() OWNER TO nitish;

--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_updated_at_column() OWNER TO postgres;

--
-- Name: validate_csrf_token(character varying, uuid); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.validate_csrf_token(p_token character varying, p_session_id uuid) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_valid BOOLEAN;
BEGIN
    SELECT EXISTS(
        SELECT 1 FROM csrf_tokens
        WHERE token = p_token
          AND session_id = p_session_id
          AND expires_at > NOW()
    ) INTO v_valid;
    
    RETURN v_valid;
END;
$$;


ALTER FUNCTION public.validate_csrf_token(p_token character varying, p_session_id uuid) OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO nitish;

--
-- Name: answer_media; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.answer_media (
    id integer NOT NULL,
    answer_id integer NOT NULL,
    media_type character varying(20) NOT NULL,
    file_path text NOT NULL,
    original_filename character varying(255),
    mime_type character varying(100),
    file_size bigint,
    width integer,
    height integer,
    duration integer,
    thumbnail_path text,
    display_order integer DEFAULT 0,
    uploaded_at timestamp with time zone DEFAULT now(),
    CONSTRAINT answer_media_media_type_check CHECK (((media_type)::text = ANY (ARRAY[('image'::character varying)::text, ('video'::character varying)::text, ('file'::character varying)::text])))
);


ALTER TABLE public.answer_media OWNER TO nitish;

--
-- Name: answer_media_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.answer_media_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.answer_media_id_seq OWNER TO nitish;

--
-- Name: answer_media_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.answer_media_id_seq OWNED BY public.answer_media.id;


--
-- Name: answers; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.answers (
    id integer NOT NULL,
    question_id integer NOT NULL,
    user_id integer NOT NULL,
    content_raw text NOT NULL,
    content_rendered_html text NOT NULL,
    content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    echo_count integer DEFAULT 0,
    comment_count integer DEFAULT 0,
    search_vector tsvector,
    is_spam boolean DEFAULT false NOT NULL,
    slug character varying(255) NOT NULL,
    CONSTRAINT answers_content_raw_check CHECK ((char_length(content_raw) <= 30000))
);


ALTER TABLE public.answers OWNER TO nitish;

--
-- Name: answers_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.answers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.answers_id_seq OWNER TO nitish;

--
-- Name: answers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.answers_id_seq OWNED BY public.answers.id;


--
-- Name: comment_helpful; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.comment_helpful (
    id integer NOT NULL,
    comment_id integer NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.comment_helpful OWNER TO nitish;

--
-- Name: comment_helpful_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.comment_helpful_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comment_helpful_id_seq OWNER TO nitish;

--
-- Name: comment_helpful_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.comment_helpful_id_seq OWNED BY public.comment_helpful.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.comments (
    id integer NOT NULL,
    user_id integer NOT NULL,
    post_id integer,
    question_id integer,
    answer_id integer,
    parent_comment_id integer,
    depth_level integer DEFAULT 0,
    content_raw text NOT NULL,
    content_rendered_html text NOT NULL,
    content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    helpful_count integer DEFAULT 0,
    reply_count integer DEFAULT 0,
    CONSTRAINT comments_depth_level_check CHECK ((depth_level <= 3)),
    CONSTRAINT only_one_parent CHECK ((((post_id IS NOT NULL) AND (question_id IS NULL) AND (answer_id IS NULL)) OR ((post_id IS NULL) AND (question_id IS NOT NULL) AND (answer_id IS NULL)) OR ((post_id IS NULL) AND (question_id IS NULL) AND (answer_id IS NOT NULL))))
);


ALTER TABLE public.comments OWNER TO nitish;

--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.comments_id_seq OWNER TO nitish;

--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;


--
-- Name: csrf_tokens; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.csrf_tokens (
    id integer NOT NULL,
    session_id uuid NOT NULL,
    token character varying(64) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


ALTER TABLE public.csrf_tokens OWNER TO nitish;

--
-- Name: csrf_tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.csrf_tokens_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.csrf_tokens_id_seq OWNER TO nitish;

--
-- Name: csrf_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.csrf_tokens_id_seq OWNED BY public.csrf_tokens.id;


--
-- Name: echos; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.echos (
    id integer NOT NULL,
    user_id integer NOT NULL,
    post_id integer,
    question_id integer,
    answer_id integer,
    refract_id integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT only_one_target CHECK ((((post_id IS NOT NULL) AND (question_id IS NULL) AND (answer_id IS NULL) AND (refract_id IS NULL)) OR ((post_id IS NULL) AND (question_id IS NOT NULL) AND (answer_id IS NULL) AND (refract_id IS NULL)) OR ((post_id IS NULL) AND (question_id IS NULL) AND (answer_id IS NOT NULL) AND (refract_id IS NULL)) OR ((post_id IS NULL) AND (question_id IS NULL) AND (answer_id IS NULL) AND (refract_id IS NOT NULL))))
);


ALTER TABLE public.echos OWNER TO nitish;

--
-- Name: echos_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.echos_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.echos_id_seq OWNER TO nitish;

--
-- Name: echos_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.echos_id_seq OWNED BY public.echos.id;


--
-- Name: post_media; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.post_media (
    id integer NOT NULL,
    post_id integer NOT NULL,
    media_type character varying(20) NOT NULL,
    file_path text NOT NULL,
    original_filename character varying(255),
    mime_type character varying(100),
    file_size bigint,
    width integer,
    height integer,
    duration integer,
    thumbnail_path text,
    display_order integer DEFAULT 0,
    uploaded_at timestamp with time zone DEFAULT now(),
    CONSTRAINT post_media_media_type_check CHECK (((media_type)::text = ANY (ARRAY[('image'::character varying)::text, ('video'::character varying)::text, ('file'::character varying)::text])))
);


ALTER TABLE public.post_media OWNER TO nitish;

--
-- Name: post_media_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.post_media_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_media_id_seq OWNER TO nitish;

--
-- Name: post_media_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.post_media_id_seq OWNED BY public.post_media.id;


--
-- Name: post_tags; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.post_tags (
    id integer NOT NULL,
    post_id integer NOT NULL,
    tag_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.post_tags OWNER TO nitish;

--
-- Name: post_tags_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.post_tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_tags_id_seq OWNER TO nitish;

--
-- Name: post_tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.post_tags_id_seq OWNED BY public.post_tags.id;


--
-- Name: posts; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.posts (
    id integer NOT NULL,
    user_id integer NOT NULL,
    title character varying(300),
    content_raw text NOT NULL,
    content_rendered_html text NOT NULL,
    content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    echo_count integer DEFAULT 0,
    refract_count integer DEFAULT 0,
    comment_count integer DEFAULT 0,
    search_vector tsvector,
    slug character varying(255) NOT NULL,
    is_spam boolean DEFAULT false NOT NULL,
    CONSTRAINT posts_content_raw_check CHECK ((char_length(content_raw) <= 30000))
);


ALTER TABLE public.posts OWNER TO nitish;

--
-- Name: posts_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.posts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.posts_id_seq OWNER TO nitish;

--
-- Name: posts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.posts_id_seq OWNED BY public.posts.id;


--
-- Name: question_media; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.question_media (
    id integer NOT NULL,
    question_id integer NOT NULL,
    media_type character varying(20) NOT NULL,
    file_path text NOT NULL,
    original_filename character varying(255),
    mime_type character varying(100),
    file_size bigint,
    width integer,
    height integer,
    duration integer,
    thumbnail_path text,
    display_order integer DEFAULT 0,
    uploaded_at timestamp with time zone DEFAULT now(),
    CONSTRAINT question_media_media_type_check CHECK (((media_type)::text = ANY (ARRAY[('image'::character varying)::text, ('video'::character varying)::text, ('file'::character varying)::text])))
);


ALTER TABLE public.question_media OWNER TO nitish;

--
-- Name: question_media_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.question_media_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.question_media_id_seq OWNER TO nitish;

--
-- Name: question_media_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.question_media_id_seq OWNED BY public.question_media.id;


--
-- Name: question_tags; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.question_tags (
    id integer NOT NULL,
    question_id integer NOT NULL,
    tag_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.question_tags OWNER TO nitish;

--
-- Name: question_tags_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.question_tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.question_tags_id_seq OWNER TO nitish;

--
-- Name: question_tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.question_tags_id_seq OWNED BY public.question_tags.id;


--
-- Name: questions; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.questions (
    id integer NOT NULL,
    user_id integer NOT NULL,
    title character varying(300) NOT NULL,
    content_raw text NOT NULL,
    content_rendered_html text NOT NULL,
    content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    echo_count integer DEFAULT 0,
    answer_count integer DEFAULT 0,
    comment_count integer DEFAULT 0,
    search_vector tsvector,
    slug character varying(255) NOT NULL,
    is_spam boolean DEFAULT false NOT NULL,
    CONSTRAINT questions_content_raw_check CHECK ((char_length(content_raw) <= 15000))
);


ALTER TABLE public.questions OWNER TO nitish;

--
-- Name: questions_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.questions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.questions_id_seq OWNER TO nitish;

--
-- Name: questions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.questions_id_seq OWNED BY public.questions.id;


--
-- Name: rate_limits; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.rate_limits (
    id integer NOT NULL,
    user_id integer DEFAULT 0 NOT NULL,
    ip_address inet,
    action_type character varying(50) NOT NULL,
    window_type character varying(20) NOT NULL,
    window_start timestamp with time zone NOT NULL,
    count integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT check_user_or_ip CHECK (((user_id > 0) OR (ip_address IS NOT NULL)))
);


ALTER TABLE public.rate_limits OWNER TO nitish;

--
-- Name: rate_limits_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.rate_limits_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.rate_limits_id_seq OWNER TO nitish;

--
-- Name: rate_limits_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.rate_limits_id_seq OWNED BY public.rate_limits.id;


--
-- Name: refracts; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.refracts (
    id integer NOT NULL,
    user_id integer NOT NULL,
    original_post_id integer NOT NULL,
    content_raw text NOT NULL,
    content_rendered_html text NOT NULL,
    content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    echo_count integer DEFAULT 0,
    is_spam boolean DEFAULT false NOT NULL,
    CONSTRAINT refracts_content_raw_check CHECK ((char_length(content_raw) <= 1500))
);


ALTER TABLE public.refracts OWNER TO nitish;

--
-- Name: refracts_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.refracts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.refracts_id_seq OWNER TO nitish;

--
-- Name: refracts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.refracts_id_seq OWNED BY public.refracts.id;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id integer NOT NULL,
    ip_address inet,
    user_agent text,
    last_used_at timestamp with time zone DEFAULT now(),
    rotated_from uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


ALTER TABLE public.sessions OWNER TO nitish;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.tags (
    id integer NOT NULL,
    name character varying(35) NOT NULL,
    slug character varying(35) NOT NULL,
    usage_count integer DEFAULT 0,
    follower_count integer DEFAULT 0,
    created_by_user_id integer,
    created_at timestamp with time zone DEFAULT now(),
    is_active boolean DEFAULT true
);


ALTER TABLE public.tags OWNER TO nitish;

--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tags_id_seq OWNER TO nitish;

--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.tags_id_seq OWNED BY public.tags.id;


--
-- Name: user_links; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.user_links (
    id integer NOT NULL,
    user_id integer NOT NULL,
    platform character varying(50) NOT NULL,
    url character varying(500) NOT NULL,
    display_text character varying(100),
    display_order integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.user_links OWNER TO nitish;

--
-- Name: user_links_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.user_links_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.user_links_id_seq OWNER TO nitish;

--
-- Name: user_links_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.user_links_id_seq OWNED BY public.user_links.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: nitish
--

CREATE TABLE public.users (
    id integer NOT NULL,
    github_id bigint,
    github_username character varying(255),
    username character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    avatar_url text,
    bio_raw text,
    bio_rendered_html text,
    bio_content_hash character varying(64),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    last_username_changed_at timestamp with time zone
);


ALTER TABLE public.users OWNER TO nitish;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: nitish
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO nitish;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: nitish
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: answer_media id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answer_media ALTER COLUMN id SET DEFAULT nextval('public.answer_media_id_seq'::regclass);


--
-- Name: answers id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answers ALTER COLUMN id SET DEFAULT nextval('public.answers_id_seq'::regclass);


--
-- Name: comment_helpful id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comment_helpful ALTER COLUMN id SET DEFAULT nextval('public.comment_helpful_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);


--
-- Name: csrf_tokens id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.csrf_tokens ALTER COLUMN id SET DEFAULT nextval('public.csrf_tokens_id_seq'::regclass);


--
-- Name: echos id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos ALTER COLUMN id SET DEFAULT nextval('public.echos_id_seq'::regclass);


--
-- Name: post_media id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_media ALTER COLUMN id SET DEFAULT nextval('public.post_media_id_seq'::regclass);


--
-- Name: post_tags id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_tags ALTER COLUMN id SET DEFAULT nextval('public.post_tags_id_seq'::regclass);


--
-- Name: posts id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.posts ALTER COLUMN id SET DEFAULT nextval('public.posts_id_seq'::regclass);


--
-- Name: question_media id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_media ALTER COLUMN id SET DEFAULT nextval('public.question_media_id_seq'::regclass);


--
-- Name: question_tags id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_tags ALTER COLUMN id SET DEFAULT nextval('public.question_tags_id_seq'::regclass);


--
-- Name: questions id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.questions ALTER COLUMN id SET DEFAULT nextval('public.questions_id_seq'::regclass);


--
-- Name: rate_limits id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.rate_limits ALTER COLUMN id SET DEFAULT nextval('public.rate_limits_id_seq'::regclass);


--
-- Name: refracts id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.refracts ALTER COLUMN id SET DEFAULT nextval('public.refracts_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);


--
-- Name: user_links id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.user_links ALTER COLUMN id SET DEFAULT nextval('public.user_links_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: answer_media answer_media_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answer_media
    ADD CONSTRAINT answer_media_pkey PRIMARY KEY (id);


--
-- Name: answers answers_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answers
    ADD CONSTRAINT answers_pkey PRIMARY KEY (id);


--
-- Name: answers answers_slug_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answers
    ADD CONSTRAINT answers_slug_key UNIQUE (slug);


--
-- Name: comment_helpful comment_helpful_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comment_helpful
    ADD CONSTRAINT comment_helpful_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: csrf_tokens csrf_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_pkey PRIMARY KEY (id);


--
-- Name: csrf_tokens csrf_tokens_token_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_token_key UNIQUE (token);


--
-- Name: echos echos_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_pkey PRIMARY KEY (id);


--
-- Name: post_media post_media_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_media
    ADD CONSTRAINT post_media_pkey PRIMARY KEY (id);


--
-- Name: post_tags post_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_pkey PRIMARY KEY (id);


--
-- Name: posts posts_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_pkey PRIMARY KEY (id);


--
-- Name: posts posts_slug_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_slug_key UNIQUE (slug);


--
-- Name: question_media question_media_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_media
    ADD CONSTRAINT question_media_pkey PRIMARY KEY (id);


--
-- Name: question_tags question_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_tags
    ADD CONSTRAINT question_tags_pkey PRIMARY KEY (id);


--
-- Name: questions questions_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.questions
    ADD CONSTRAINT questions_pkey PRIMARY KEY (id);


--
-- Name: questions questions_slug_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.questions
    ADD CONSTRAINT questions_slug_key UNIQUE (slug);


--
-- Name: rate_limits rate_limits_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.rate_limits
    ADD CONSTRAINT rate_limits_pkey PRIMARY KEY (id);


--
-- Name: refracts refracts_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.refracts
    ADD CONSTRAINT refracts_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: tags tags_name_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_name_key UNIQUE (name);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: tags tags_slug_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_slug_key UNIQUE (slug);


--
-- Name: comment_helpful unique_comment_helpful; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comment_helpful
    ADD CONSTRAINT unique_comment_helpful UNIQUE (comment_id, user_id);


--
-- Name: post_tags unique_post_tag; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT unique_post_tag UNIQUE (post_id, tag_id);


--
-- Name: question_tags unique_question_tag; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_tags
    ADD CONSTRAINT unique_question_tag UNIQUE (question_id, tag_id);


--
-- Name: echos unique_user_answer_echo; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT unique_user_answer_echo UNIQUE (user_id, answer_id);


--
-- Name: user_links unique_user_platform; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.user_links
    ADD CONSTRAINT unique_user_platform UNIQUE (user_id, platform);


--
-- Name: echos unique_user_post_echo; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT unique_user_post_echo UNIQUE (user_id, post_id);


--
-- Name: refracts unique_user_post_refract; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.refracts
    ADD CONSTRAINT unique_user_post_refract UNIQUE (user_id, original_post_id);


--
-- Name: echos unique_user_question_echo; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT unique_user_question_echo UNIQUE (user_id, question_id);


--
-- Name: echos unique_user_refract_echo; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT unique_user_refract_echo UNIQUE (user_id, refract_id);


--
-- Name: user_links user_links_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.user_links
    ADD CONSTRAINT user_links_pkey PRIMARY KEY (id);


--
-- Name: users users_github_id_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_github_id_key UNIQUE (github_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: answers_slug_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX answers_slug_idx ON public.answers USING btree (slug);


--
-- Name: answers_spam_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX answers_spam_idx ON public.answers USING btree (is_spam) WHERE (is_spam = false);


--
-- Name: idx_answer_media_answer_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answer_media_answer_id ON public.answer_media USING btree (answer_id);


--
-- Name: idx_answers_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answers_created_at ON public.answers USING btree (created_at DESC);


--
-- Name: idx_answers_echo_count; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answers_echo_count ON public.answers USING btree (echo_count DESC);


--
-- Name: idx_answers_question_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answers_question_id ON public.answers USING btree (question_id);


--
-- Name: idx_answers_search_vector; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answers_search_vector ON public.answers USING gin (search_vector);


--
-- Name: idx_answers_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_answers_user_id ON public.answers USING btree (user_id);


--
-- Name: idx_comment_helpful_comment_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comment_helpful_comment_id ON public.comment_helpful USING btree (comment_id);


--
-- Name: idx_comment_helpful_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comment_helpful_user_id ON public.comment_helpful USING btree (user_id);


--
-- Name: idx_comments_answer_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_answer_id ON public.comments USING btree (answer_id);


--
-- Name: idx_comments_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_created_at ON public.comments USING btree (created_at DESC);


--
-- Name: idx_comments_parent_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_parent_id ON public.comments USING btree (parent_comment_id);


--
-- Name: idx_comments_post_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_post_id ON public.comments USING btree (post_id);


--
-- Name: idx_comments_question_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_question_id ON public.comments USING btree (question_id);


--
-- Name: idx_comments_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_comments_user_id ON public.comments USING btree (user_id);


--
-- Name: idx_csrf_tokens_expires_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_csrf_tokens_expires_at ON public.csrf_tokens USING btree (expires_at);


--
-- Name: idx_csrf_tokens_session_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_csrf_tokens_session_id ON public.csrf_tokens USING btree (session_id);


--
-- Name: idx_csrf_tokens_token; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_csrf_tokens_token ON public.csrf_tokens USING btree (token);


--
-- Name: idx_echos_answer_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_answer_id ON public.echos USING btree (answer_id);


--
-- Name: idx_echos_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_created_at ON public.echos USING btree (created_at DESC);


--
-- Name: idx_echos_post_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_post_id ON public.echos USING btree (post_id);


--
-- Name: idx_echos_question_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_question_id ON public.echos USING btree (question_id);


--
-- Name: idx_echos_refract_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_refract_id ON public.echos USING btree (refract_id);


--
-- Name: idx_echos_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_echos_user_id ON public.echos USING btree (user_id);


--
-- Name: idx_post_media_post_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_post_media_post_id ON public.post_media USING btree (post_id);


--
-- Name: idx_post_tags_post_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_post_tags_post_id ON public.post_tags USING btree (post_id);


--
-- Name: idx_post_tags_tag_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_post_tags_tag_id ON public.post_tags USING btree (tag_id);


--
-- Name: idx_posts_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_posts_created_at ON public.posts USING btree (created_at DESC);


--
-- Name: idx_posts_echo_count; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_posts_echo_count ON public.posts USING btree (echo_count DESC);


--
-- Name: idx_posts_search_vector; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_posts_search_vector ON public.posts USING gin (search_vector);


--
-- Name: idx_posts_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_posts_user_id ON public.posts USING btree (user_id);


--
-- Name: idx_question_media_question_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_question_media_question_id ON public.question_media USING btree (question_id);


--
-- Name: idx_question_tags_question_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_question_tags_question_id ON public.question_tags USING btree (question_id);


--
-- Name: idx_question_tags_tag_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_question_tags_tag_id ON public.question_tags USING btree (tag_id);


--
-- Name: idx_questions_answer_count; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_questions_answer_count ON public.questions USING btree (answer_count DESC);


--
-- Name: idx_questions_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_questions_created_at ON public.questions USING btree (created_at DESC);


--
-- Name: idx_questions_search_vector; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_questions_search_vector ON public.questions USING gin (search_vector);


--
-- Name: idx_questions_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_questions_user_id ON public.questions USING btree (user_id);


--
-- Name: idx_rate_limits_window_start; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_rate_limits_window_start ON public.rate_limits USING btree (window_start);


--
-- Name: idx_refracts_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_refracts_created_at ON public.refracts USING btree (created_at DESC);


--
-- Name: idx_refracts_echo_count; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_refracts_echo_count ON public.refracts USING btree (echo_count DESC);


--
-- Name: idx_refracts_original_post_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_refracts_original_post_id ON public.refracts USING btree (original_post_id);


--
-- Name: idx_refracts_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_refracts_user_id ON public.refracts USING btree (user_id);


--
-- Name: idx_sessions_expires_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_sessions_expires_at ON public.sessions USING btree (expires_at);


--
-- Name: idx_sessions_last_used; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_sessions_last_used ON public.sessions USING btree (last_used_at DESC);


--
-- Name: idx_sessions_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);


--
-- Name: idx_tags_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_tags_created_at ON public.tags USING btree (created_at DESC);


--
-- Name: idx_tags_name; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_tags_name ON public.tags USING btree (name);


--
-- Name: idx_tags_slug; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_tags_slug ON public.tags USING btree (slug);


--
-- Name: idx_tags_usage_count; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_tags_usage_count ON public.tags USING btree (usage_count DESC);


--
-- Name: idx_user_links_user_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_user_links_user_id ON public.user_links USING btree (user_id);


--
-- Name: idx_users_created_at; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_users_created_at ON public.users USING btree (created_at DESC);


--
-- Name: idx_users_github_id; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_users_github_id ON public.users USING btree (github_id);


--
-- Name: idx_users_username; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX idx_users_username ON public.users USING btree (username);


--
-- Name: posts_content_hash_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX posts_content_hash_idx ON public.posts USING btree (content_hash);


--
-- Name: posts_slug_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX posts_slug_idx ON public.posts USING btree (slug);


--
-- Name: posts_spam_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX posts_spam_idx ON public.posts USING btree (is_spam) WHERE (is_spam = false);


--
-- Name: questions_slug_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX questions_slug_idx ON public.questions USING btree (slug);


--
-- Name: questions_spam_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX questions_spam_idx ON public.questions USING btree (is_spam) WHERE (is_spam = false);


--
-- Name: refracts_spam_idx; Type: INDEX; Schema: public; Owner: nitish
--

CREATE INDEX refracts_spam_idx ON public.refracts USING btree (is_spam) WHERE (is_spam = false);


--
-- Name: unique_ip_rate_limit; Type: INDEX; Schema: public; Owner: nitish
--

CREATE UNIQUE INDEX unique_ip_rate_limit ON public.rate_limits USING btree (ip_address, action_type, window_type, window_start) WHERE ((ip_address IS NOT NULL) AND (user_id = 0));


--
-- Name: unique_user_rate_limit; Type: INDEX; Schema: public; Owner: nitish
--

CREATE UNIQUE INDEX unique_user_rate_limit ON public.rate_limits USING btree (user_id, action_type, window_type, window_start) WHERE ((user_id > 0) AND (ip_address IS NULL));


--
-- Name: answers trigger_answers_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_answers_updated_at BEFORE UPDATE ON public.answers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: comments trigger_comments_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_comments_updated_at BEFORE UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: comments trigger_decrement_answer_comment_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_answer_comment_count AFTER UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.decrement_answer_comment_count();


--
-- Name: comment_helpful trigger_decrement_comment_helpful_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_comment_helpful_count AFTER DELETE ON public.comment_helpful FOR EACH ROW EXECUTE FUNCTION public.decrement_comment_helpful_count();


--
-- Name: comments trigger_decrement_post_comment_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_post_comment_count AFTER UPDATE ON public.comments FOR EACH ROW EXECUTE FUNCTION public.decrement_post_comment_count();


--
-- Name: refracts trigger_decrement_post_refract_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_post_refract_count AFTER UPDATE ON public.refracts FOR EACH ROW EXECUTE FUNCTION public.decrement_post_refract_count();


--
-- Name: answers trigger_decrement_question_answer_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_question_answer_count AFTER UPDATE ON public.answers FOR EACH ROW EXECUTE FUNCTION public.decrement_question_answer_count();


--
-- Name: post_tags trigger_decrement_tag_usage_post; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_tag_usage_post AFTER DELETE ON public.post_tags FOR EACH ROW EXECUTE FUNCTION public.decrement_tag_usage_count();


--
-- Name: question_tags trigger_decrement_tag_usage_question; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_decrement_tag_usage_question AFTER DELETE ON public.question_tags FOR EACH ROW EXECUTE FUNCTION public.decrement_tag_usage_count();


--
-- Name: comments trigger_increment_answer_comment_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_answer_comment_count AFTER INSERT ON public.comments FOR EACH ROW WHEN ((new.answer_id IS NOT NULL)) EXECUTE FUNCTION public.increment_answer_comment_count();


--
-- Name: echos trigger_increment_answer_echo_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_answer_echo_count AFTER INSERT ON public.echos FOR EACH ROW WHEN ((new.answer_id IS NOT NULL)) EXECUTE FUNCTION public.increment_answer_echo_count();


--
-- Name: comment_helpful trigger_increment_comment_helpful_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_comment_helpful_count AFTER INSERT ON public.comment_helpful FOR EACH ROW EXECUTE FUNCTION public.increment_comment_helpful_count();


--
-- Name: comments trigger_increment_comment_reply_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_comment_reply_count AFTER INSERT ON public.comments FOR EACH ROW WHEN ((new.parent_comment_id IS NOT NULL)) EXECUTE FUNCTION public.increment_comment_reply_count();


--
-- Name: comments trigger_increment_post_comment_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_post_comment_count AFTER INSERT ON public.comments FOR EACH ROW WHEN ((new.post_id IS NOT NULL)) EXECUTE FUNCTION public.increment_post_comment_count();


--
-- Name: echos trigger_increment_post_echo_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_post_echo_count AFTER INSERT ON public.echos FOR EACH ROW WHEN ((new.post_id IS NOT NULL)) EXECUTE FUNCTION public.increment_post_echo_count();


--
-- Name: refracts trigger_increment_post_refract_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_post_refract_count AFTER INSERT ON public.refracts FOR EACH ROW EXECUTE FUNCTION public.increment_post_refract_count();


--
-- Name: answers trigger_increment_question_answer_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_question_answer_count AFTER INSERT ON public.answers FOR EACH ROW EXECUTE FUNCTION public.increment_question_answer_count();


--
-- Name: comments trigger_increment_question_comment_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_question_comment_count AFTER INSERT ON public.comments FOR EACH ROW WHEN ((new.question_id IS NOT NULL)) EXECUTE FUNCTION public.increment_question_comment_count();


--
-- Name: echos trigger_increment_question_echo_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_question_echo_count AFTER INSERT ON public.echos FOR EACH ROW WHEN ((new.question_id IS NOT NULL)) EXECUTE FUNCTION public.increment_question_echo_count();


--
-- Name: echos trigger_increment_refract_echo_count; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_refract_echo_count AFTER INSERT ON public.echos FOR EACH ROW WHEN ((new.refract_id IS NOT NULL)) EXECUTE FUNCTION public.increment_refract_echo_count();


--
-- Name: post_tags trigger_increment_tag_usage_post; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_tag_usage_post AFTER INSERT ON public.post_tags FOR EACH ROW EXECUTE FUNCTION public.increment_tag_usage_count();


--
-- Name: question_tags trigger_increment_tag_usage_question; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_increment_tag_usage_question AFTER INSERT ON public.question_tags FOR EACH ROW EXECUTE FUNCTION public.increment_tag_usage_count();


--
-- Name: posts trigger_posts_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_posts_updated_at BEFORE UPDATE ON public.posts FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: questions trigger_questions_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_questions_updated_at BEFORE UPDATE ON public.questions FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: refracts trigger_refracts_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_refracts_updated_at BEFORE UPDATE ON public.refracts FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: answers trigger_update_answer_search_vector; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_update_answer_search_vector BEFORE INSERT OR UPDATE OF content_raw ON public.answers FOR EACH ROW EXECUTE FUNCTION public.update_answer_search_vector();


--
-- Name: posts trigger_update_post_search_vector; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_update_post_search_vector BEFORE INSERT OR UPDATE OF title, content_raw ON public.posts FOR EACH ROW EXECUTE FUNCTION public.update_post_search_vector();


--
-- Name: questions trigger_update_question_search_vector; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_update_question_search_vector BEFORE INSERT OR UPDATE OF title, content_raw ON public.questions FOR EACH ROW EXECUTE FUNCTION public.update_question_search_vector();


--
-- Name: users trigger_users_updated_at; Type: TRIGGER; Schema: public; Owner: nitish
--

CREATE TRIGGER trigger_users_updated_at BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: answer_media answer_media_answer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answer_media
    ADD CONSTRAINT answer_media_answer_id_fkey FOREIGN KEY (answer_id) REFERENCES public.answers(id) ON DELETE CASCADE;


--
-- Name: answers answers_question_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answers
    ADD CONSTRAINT answers_question_id_fkey FOREIGN KEY (question_id) REFERENCES public.questions(id) ON DELETE CASCADE;


--
-- Name: answers answers_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.answers
    ADD CONSTRAINT answers_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: comment_helpful comment_helpful_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comment_helpful
    ADD CONSTRAINT comment_helpful_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON DELETE CASCADE;


--
-- Name: comment_helpful comment_helpful_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comment_helpful
    ADD CONSTRAINT comment_helpful_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: comments comments_answer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_answer_id_fkey FOREIGN KEY (answer_id) REFERENCES public.answers(id) ON DELETE CASCADE;


--
-- Name: comments comments_parent_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_parent_comment_id_fkey FOREIGN KEY (parent_comment_id) REFERENCES public.comments(id) ON DELETE CASCADE;


--
-- Name: comments comments_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: comments comments_question_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_question_id_fkey FOREIGN KEY (question_id) REFERENCES public.questions(id) ON DELETE CASCADE;


--
-- Name: comments comments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: csrf_tokens csrf_tokens_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.sessions(id) ON DELETE CASCADE;


--
-- Name: echos echos_answer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_answer_id_fkey FOREIGN KEY (answer_id) REFERENCES public.answers(id) ON DELETE CASCADE;


--
-- Name: echos echos_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: echos echos_question_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_question_id_fkey FOREIGN KEY (question_id) REFERENCES public.questions(id) ON DELETE CASCADE;


--
-- Name: echos echos_refract_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_refract_id_fkey FOREIGN KEY (refract_id) REFERENCES public.refracts(id) ON DELETE CASCADE;


--
-- Name: echos echos_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.echos
    ADD CONSTRAINT echos_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: post_media post_media_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_media
    ADD CONSTRAINT post_media_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: post_tags post_tags_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: post_tags post_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: posts posts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: question_media question_media_question_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_media
    ADD CONSTRAINT question_media_question_id_fkey FOREIGN KEY (question_id) REFERENCES public.questions(id) ON DELETE CASCADE;


--
-- Name: question_tags question_tags_question_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_tags
    ADD CONSTRAINT question_tags_question_id_fkey FOREIGN KEY (question_id) REFERENCES public.questions(id) ON DELETE CASCADE;


--
-- Name: question_tags question_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.question_tags
    ADD CONSTRAINT question_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: questions questions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.questions
    ADD CONSTRAINT questions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: refracts refracts_original_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.refracts
    ADD CONSTRAINT refracts_original_post_id_fkey FOREIGN KEY (original_post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: refracts refracts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.refracts
    ADD CONSTRAINT refracts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tags tags_created_by_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_created_by_user_id_fkey FOREIGN KEY (created_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: user_links user_links_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nitish
--

ALTER TABLE ONLY public.user_links
    ADD CONSTRAINT user_links_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: DEFAULT PRIVILEGES FOR TABLES; Type: DEFAULT ACL; Schema: public; Owner: postgres
--

ALTER DEFAULT PRIVILEGES FOR ROLE postgres IN SCHEMA public GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLES TO nitish;


--
-- PostgreSQL database dump complete
--

\unrestrict MLHnqPH6LGZ2geRf5l1OQfgxYHz9bEaFruayJZHpKvzXgOKgTwOrPfQrLZJJzAa

